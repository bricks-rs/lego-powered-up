//! # lego-powered-up
//! This is a crate for remotely controlling Lego PoweredUp hubs and devices
//! over Bluetooth. It provides a simple synchronous interface, with the
//! intention that arbitrarily complex programs may be written in Rust.
//!
//! ## Example
//! ```no_run
//! use lego_powered_up::notifications::Power;
//! use lego_powered_up::PoweredUp;
//! use std::{thread::sleep, time::Duration};
//!
//! fn main() -> anyhow::Result<()> {
//!     println!("Listening for hubs...");
//!     let pu = PoweredUp::init()?;
//!     let hub = pu.wait_for_hub()?;
//!
//!     println!("Connecting to hub `{}`", hub.name);
//!     let hub = pu.create_hub(&hub)?;
//!
//!     println!("Change the hub LED to green");
//!     let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed)?;
//!     hub_led.set_rgb(&[0, 0xff, 0])?;
//!
//!     println!("Run motors");
//!     let mut motor_c = hub.port(lego_powered_up::hubs::Port::C)?;
//!     let mut motor_d = hub.port(lego_powered_up::hubs::Port::D)?;
//!     motor_c.start_speed(50, Power::Cw(50))?;
//!     motor_d.start_speed(50, Power::Cw(50))?;
//!
//!     sleep(Duration::from_secs(3));
//!
//!     println!("Stop motors");
//!     motor_c.start_speed(0, Power::Float)?;
//!     motor_d.start_speed(0, Power::Brake)?;
//!
//!     println!("Disconnect from hub `{}`", hub.get_name());
//!     hub.disconnect()?;
//!
//!     println!("Done!");
//!
//!     Ok(())
//! }
//! ```

use crate::devices::{create_device, Device};
use crate::hubs::ConnectedIo;
use anyhow::{anyhow, bail, Context, Result};
pub use btleplug::api::{BDAddr, Peripheral};
use btleplug::api::{Central, CentralEvent, Characteristic};
use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread::{self, sleep};
use std::time::Duration;

#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, manager::Manager};

#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};

#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};

#[allow(unused)]
use log::{debug, error, info, trace, warn};

use consts::*;
use hubs::Port;
use notifications::{AttachedIo, NotificationMessage};

#[allow(unused)]
pub mod consts;

pub mod devices;
pub mod hubs;
pub mod notifications;

#[cfg(target_os = "linux")]
pub fn print_adapter_info(idx: usize, adapter: &Adapter) -> Result<()> {
    /*info!(
        "connected adapter {:?} is powered: {:?}",
        adapter.name(),
        adapter.is_powered()
    );*/
    println!("  {}: {}", idx, adapter.name()?);
    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn print_adapter_info(idx: usize, _adapter: &Adapter) -> Result<()> {
    info!("adapter info can't be printed on Windows 10 or mac");
    println!("  {}: Adapter {}", idx, idx);
    Ok(())
}

/// This is the main interface for interacting with the background process
/// which manages the state.
pub struct PoweredUp {
    // _manager: Manager,
    adapter: Arc<RwLock<Adapter>>,
    control_tx: Option<Sender<PoweredUpInternalControlMessage>>,
    hub_manager_tx: Option<Sender<HubManagerMessage>>,
    pub hubs: Vec<Box<dyn Hub + Send + Sync>>,
}

impl PoweredUp {
    /// List the available BLE adapters
    pub fn devices() -> Result<Vec<Adapter>> {
        let manager = Manager::new()?;
        Ok(manager.adapters()?)
    }

    /// Initialise a PoweredUp connection using the first available BLE
    /// adapter
    pub fn init() -> Result<Self> {
        Self::with_device(0)
    }

    /// Initialise a PoweredUp connectin using the `dev`th adapter. Adapter
    /// indices may be obtained via the `PoweredUp::devices` function.
    ///
    /// Launches the background worker thread.
    pub fn with_device(dev: usize) -> Result<Self> {
        let manager = Manager::new()?;
        let adapters = manager.adapters()?;
        let adapter =
            adapters.into_iter().nth(dev).context("No adapter found")?;

        let mut pu = Self {
            //   _manager: manager,
            adapter: Arc::new(RwLock::new(adapter)),
            control_tx: None,
            hub_manager_tx: None,
            hubs: Vec::new(),
        };
        pu.run()?;

        Ok(pu)
    }

    /// Launch the background worker thread
    fn run(&mut self) -> Result<()> {
        let event_rx = self
            .adapter
            .write()
            .unwrap()
            .event_receiver()
            .context("Unable to access event receiver")?;
        let mut worker = PoweredUpInternal::new(self.adapter.clone());

        let (control_tx, control_rx) = bounded(10);

        thread::spawn(move || {
            worker.run(control_rx, event_rx).unwrap();
        });

        self.control_tx = Some(control_tx);

        let (hm_tx, hm_rx) = unbounded();
        self.hub_manager_tx = Some(hm_tx.clone());
        let adapter_clone = self.adapter.clone();
        thread::spawn(move || {
            HubManager::run(adapter_clone, hm_rx, hm_tx).unwrap();
        });

        self.adapter.write().unwrap().start_scan()?;

        Ok(())
    }

    /// Stop the background worker thread
    pub fn stop(&mut self) -> Result<()> {
        if let Some(tx) = &self.control_tx {
            tx.send(PoweredUpInternalControlMessage::Stop)?;
        }
        Ok(())
    }

    pub fn peripheral(&self, dev: BDAddr) -> Option<impl Peripheral> {
        self.adapter.write().unwrap().peripheral(dev)
    }

    /// Connect to a discovered hub, returning a `HubController` instance to
    /// communicate with it.
    pub fn create_hub(&self, hub: &DiscoveredHub) -> Result<HubController> {
        let retries: usize = 10;
        for idx in 1..=retries {
            info!(
                "Connecting to hub {} attempt {} of {}...",
                hub.addr, idx, retries
            );
            let (resp_tx, resp_rx) = bounded(1);
            self.hub_manager_tx
                .as_ref()
                .unwrap()
                .send(HubManagerMessage::ConnectToHub(hub.clone(), resp_tx))?;
            match resp_rx.recv()? {
                Ok(controller) => return Ok(controller),
                Err(e) => warn!("{}", e),
            }
            sleep(Duration::from_secs(3));
        }
        Err(anyhow!(
            "Unable to connect to {} after {} tries",
            hub.addr,
            retries
        ))
    }

    /// Connect to a specific hub by BLE address
    pub fn connect_to_hub(&self, addr: &str) -> Result<HubController> {
        let dh = DiscoveredHub {
            hub_type: HubType::Unknown,
            addr: BDAddr::from_str(addr)?,
            name: Default::default(),
        };

        self.create_hub(&dh)
    }

    /// Listen for hub announcements and return a description of the first
    /// discovered device (which may be passed to `create_hub`)
    pub fn wait_for_hub(&self) -> Result<DiscoveredHub> {
        let timeout = Duration::from_secs(9999);
        self.wait_for_hub_filter_timeout_internal(None, timeout)
    }

    /// Listen for a hub matching the provided filter, returning a description
    /// suitable for passing to `create_hub`
    pub fn wait_for_hub_filter(
        &self,
        filter: HubFilter,
    ) -> Result<DiscoveredHub> {
        let timeout = Duration::from_secs(9999);
        self.wait_for_hub_filter_timeout_internal(Some(filter), timeout)
    }

    /// Listen for a hub matching the provided filter, waiting up to the
    /// provided timeout, returning a description suitable for passing to
    /// `create_hub`
    pub fn wait_for_hub_filter_timeout(
        &self,
        filter: HubFilter,
        timeout: Duration,
    ) -> Result<DiscoveredHub> {
        self.wait_for_hub_filter_timeout_internal(Some(filter), timeout)
    }

    fn wait_for_hub_filter_timeout_internal(
        &self,
        filter: Option<HubFilter>,
        timeout: Duration,
    ) -> Result<DiscoveredHub> {
        let (tx, rx) = bounded(1);
        let params = HubNotificationParams {
            response: tx,
            filter,
        };

        self.control_tx
            .as_ref()
            .unwrap()
            .send(PoweredUpInternalControlMessage::WaitForHub(params))?;

        select! {
            recv(rx) -> msg => {
               Ok(msg?)
            }
            default(timeout) => {
                bail!("Timeout reached")
            }
        }
    }

    /// List all the hubs that this PoweredUp instance has seen announcing
    /// their existence
    pub fn list_discovered_hubs(&self) -> Result<Vec<DiscoveredHub>> {
        let (tx, rx) = bounded(1);
        self.control_tx
            .as_ref()
            .unwrap()
            .send(PoweredUpInternalControlMessage::ListDiscoveredHubs(tx))?;
        Ok(rx.recv()?)
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub(crate) enum DeviceNotificationMessage {
    HubDiscovered(DiscoveredHub),
}

/// Properties by which to filter discovered hubs
#[derive(Debug)]
pub enum HubFilter {
    /// Hub name must match the provided value
    Name(String),
    /// Hub address must match the provided value
    Addr(String),
}

impl HubFilter {
    /// Test whether the discovered hub matches the provided filter mode
    pub fn matches(&self, hub: &DiscoveredHub) -> bool {
        use HubFilter::*;
        match self {
            Name(n) => hub.name == *n,
            Addr(a) => hub.addr.to_string() == *a,
        }
    }
}

/// Struct describing a discovered hub. This description may be passed
/// to `PoweredUp::create_hub` to initialise a connection.
#[derive(Clone, Debug)]
pub struct DiscoveredHub {
    /// Type of hub, e.g. TechnicMediumHub
    pub hub_type: HubType,
    /// BLE address
    pub addr: BDAddr,
    /// Friendly name of the hub, as set in the PoweredUp/Control+ apps
    pub name: String,
}

#[derive(Debug)]
enum PoweredUpInternalControlMessage {
    Stop,
    WaitForHub(HubNotificationParams),
    ListDiscoveredHubs(Sender<Vec<DiscoveredHub>>),
}

#[derive(Debug)]
struct HubNotificationParams {
    response: Sender<DiscoveredHub>,
    filter: Option<HubFilter>,
}

struct PoweredUpInternal {
    adapter: Arc<RwLock<Adapter>>,
    discovered_hubs: Vec<DiscoveredHub>,
    hub_notifications: Option<HubNotificationParams>,
}

impl PoweredUpInternal {
    pub fn new(adapter: Arc<RwLock<Adapter>>) -> Self {
        Self {
            adapter,
            discovered_hubs: Default::default(),
            hub_notifications: None,
        }
    }
    pub fn run(
        &mut self,
        control_channel: Receiver<PoweredUpInternalControlMessage>,
        event_rx: std::sync::mpsc::Receiver<CentralEvent>,
    ) -> Result<()> {
        use DeviceNotificationMessage::*;
        info!("Starting PoweredUp connection manager");

        let (device_notification_sender, device_notification_receiver) =
            bounded(16);
        let adapter_clone = self.adapter.clone();
        thread::spawn(move || {
            PoweredUpInternal::btle_notification_listener(
                event_rx,
                device_notification_sender,
                adapter_clone,
            )
        });
        loop {
            select! {
                recv(device_notification_receiver) -> msg => {
                    println!("PU INTERNAL MSG: {:?}", msg);
                    match msg.unwrap() {
                        HubDiscovered(hub) => {
                            if let Some(notify) = self.hub_notifications.take() {
                                // Take ownership of the HubNotificationParams
                                // struct because we need to own the channel to
                                // send through it.
                                let mut send_it = true;
                                if let Some(filter) = &notify.filter {
                                    if !filter.matches(&hub) {
                                        send_it = false;
                                    }
                                }
                                if send_it {
                                    // ignore the status of the send - this
                                    // will be an Err if the receiving end
                                    // has timed out
                                    let _ = notify.response.send(hub.clone());
                                } else {
                                    // If no notification was sent then put
                                    // the params struct back for next time
                                    self.hub_notifications = Some(notify);
                                }
                            }
                            self.discovered_hubs.push(hub);

                        }
                    }
                }
                recv(control_channel) -> msg => {
                    use PoweredUpInternalControlMessage::*;
                    match msg.unwrap() { // TODO disconnect all hubs
                        Stop => return Ok(()),
                        WaitForHub(params) => {
                            self.hub_notifications = Some(params);
                        }
                        ListDiscoveredHubs(response) => {
                            response.send(self.discovered_hubs.clone()).unwrap();
                        }
                    }
                }
            };
        }
    }

    fn btle_notification_listener(
        event_rx: std::sync::mpsc::Receiver<CentralEvent>,
        device_notification_sender: Sender<DeviceNotificationMessage>,
        adapter: Arc<RwLock<Adapter>>,
    ) -> ! {
        use CentralEvent::*;
        info!("Starting btleplug notification proxy");
        loop {
            let mut notification = None;
            if let Ok(evt) = event_rx.recv() {
                info!("evt: {:?}", evt);
                match evt {
                    DeviceDiscovered(dev) => {
                        let adapter = adapter.write().unwrap();
                        let peripheral = adapter.peripheral(dev).unwrap();
                        debug!(
                            "peripheral : {:?} is connected: {:?}",
                            peripheral.properties().local_name,
                            peripheral.is_connected()
                        );
                        if peripheral.properties().local_name.is_some()
                            && !peripheral.is_connected()
                        {
                            let name =
                                peripheral.properties().local_name.unwrap();
                            if let Some(hub_type) = peripheral.identify() {
                                debug!("Looks like a '{:?}' hub!", hub_type);
                                notification = Some(
                                    DeviceNotificationMessage::HubDiscovered(
                                        DiscoveredHub {
                                            hub_type,
                                            addr: dev,
                                            name,
                                        },
                                    ),
                                );
                            } else {
                                debug!(
                                    "Device does not look like a PoweredUp Hub"
                                );
                            }
                        }
                    }
                    _ => {} //TODO handle other events
                }
            } else {
                panic!("Events channel disconnected!");
            }

            if let Some(notif) = notification {
                device_notification_sender
                    .send(notif)
                    .expect("Device notification channel failed");
            }
        }
    }
}

/// Struct representing a hub. Provides methods to interact with the
/// background worker thread managing the connection
#[derive(Clone, Debug)]
pub struct HubController {
    addr: BDAddr,
    hub_type: HubType,
    name: String,
    hub_manager_tx: Sender<HubManagerMessage>,
}

impl HubController {
    /// Returns the friendly name of the hub, as set via the PoweredUp or
    /// Control+ apps
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the type of the hub, e.g. TechnicMediumHub
    pub fn get_type(&self) -> HubType {
        self.hub_type
    }

    /// Returns the BLE address of the hub
    pub fn get_addr(&self) -> &BDAddr {
        &self.addr
    }

    /// Disconnect from the hub
    pub fn disconnect(&self) -> Result<()> {
        let (tx, rx) = bounded(1);
        self.hub_manager_tx
            .send(HubManagerMessage::Disconnect(self.addr, tx))?;

        rx.recv()?
    }

    /// Get a controller handle for the specified port
    pub fn port(&self, port: Port) -> Result<PortController> {
        let (tx, rx) = bounded::<Result<PortController>>(1);
        self.hub_manager_tx
            .send(HubManagerMessage::GetPort(self.addr, port, tx))?;
        rx.recv()?
    }

    /// Enumerate the attached devices
    pub fn get_attached_io(&self) -> Result<Vec<ConnectedIo>> {
        let (tx, rx) = bounded(1);
        self.hub_manager_tx
            .send(HubManagerMessage::GetAttachedIo(self.addr, tx))?;
        rx.recv()?
    }
}

/// Struct representing a port. Provides methods to interact with the
/// background worker thread
#[derive(Debug)]
pub struct PortController {
    port_id: u8,
    port_type: Port,
    device: Box<dyn Device>,
}

impl Deref for PortController {
    type Target = Box<dyn Device + 'static>;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
impl DerefMut for PortController {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

#[derive(Debug)]
enum HubManagerMessage {
    ConnectToHub(DiscoveredHub, Sender<Result<HubController>>),
    Notification(BDAddr, NotificationMessage),
    SendToHub(BDAddr, NotificationMessage, Sender<Result<()>>),
    Disconnect(BDAddr, Sender<Result<()>>),
    GetPort(BDAddr, Port, Sender<Result<PortController>>),
    GetAttachedIo(BDAddr, Sender<Result<Vec<ConnectedIo>>>),
}

struct HubManager;

impl HubManager {
    pub fn run(
        adapter: Arc<RwLock<Adapter>>,
        command_rx: Receiver<HubManagerMessage>,
        command_tx: Sender<HubManagerMessage>,
    ) -> Result<()> {
        use HubManagerMessage::*;

        let mut hubs: HashMap<BDAddr, Box<dyn Hub + Send + Sync>> =
            Default::default();

        while let Ok(msg) = command_rx.recv() {
            debug!("HubManager: received `{:?}`", msg);
            match msg {
                ConnectToHub(hub, response) => {
                    response
                        .send(HubManager::connect_to_hub(
                            &adapter,
                            hub,
                            &mut hubs,
                            command_tx.clone(),
                        ))
                        .unwrap();
                }
                Notification(addr, msg) => {
                    info!("[{}] Received message: {:?}", addr, msg);
                    if let Some(hub) = hubs.get_mut(&addr) {
                        match msg {
                            NotificationMessage::HubAttachedIo(io) => {
                                hub.process_io_event(io);
                            }
                            _ => {}
                        }
                    } else {
                        error!("Received message for invalid hub");
                    }
                }
                GetPort(addr, port, response) => {
                    if let Some(hub) = &hubs.get(&addr) {
                        // hub exists
                        let port_map = hub.port_map();
                        if let Some(port_id) = port_map.get(&port) {
                            // create a port controller with this information
                            let device = create_device(
                                *port_id,
                                port,
                                addr,
                                command_tx.clone(),
                            );
                            let controller = PortController {
                                device,
                                port_id: *port_id,
                                port_type: port,
                            };
                            response.send(Ok(controller)).unwrap();
                        } else {
                            // chosen port does not exist on this hub
                            let m = Err(anyhow!(
                                "Port {:?} does not exist on hub {}",
                                port,
                                addr
                            ));
                            response.send(m).unwrap();
                        }
                    } else {
                        // address does not correspond to a hub
                        let m =
                            Err(anyhow!("No hub found for address {}", addr));
                        response.send(m).unwrap();
                    }
                }

                SendToHub(addr, msg, response) => {
                    if let Some(hub) = hubs.get(&addr) {
                        // hub exists - now get peripheral handle
                        let status = hub.send(msg);
                        response.send(status).unwrap();
                    } else {
                        // address does not correspond to a hub
                        let m =
                            Err(anyhow!("No hub found for address {}", addr));
                        response.send(m).unwrap();
                    }
                }
                Disconnect(addr, response) => {
                    response
                        .send(HubManager::disconnect(addr, &mut hubs))
                        .unwrap();
                }

                GetAttachedIo(addr, response) => {
                    if let Some(hub) = hubs.get(&addr) {
                        // hub exists - get the attached IO devices
                        response.send(Ok(hub.attached_io())).unwrap();
                    } else {
                        response
                            .send(Err(anyhow!(
                                "No hub found for address {}",
                                addr
                            )))
                            .unwrap();
                    }
                }
            }
        }
        Ok(())
    }

    fn connect_to_hub(
        adapter: &Arc<RwLock<Adapter>>,
        hub: DiscoveredHub,
        hubs: &mut HashMap<BDAddr, Box<dyn Hub + Send + Sync>>,
        command_tx: Sender<HubManagerMessage>,
    ) -> Result<HubController> {
        let peripheral =
            adapter.write().unwrap().peripheral(hub.addr).context("")?;

        peripheral.connect()?;
        let chars = peripheral.discover_characteristics()?;

        let (hub_type, name) = if hub.hub_type == HubType::Unknown {
            // discover the type
            let hub_type = peripheral.identify().unwrap_or(HubType::Unknown);
            let name = peripheral.properties().local_name.unwrap_or_default();
            (hub_type, name)
        } else {
            // trust the provided type
            (hub.hub_type, hub.name)
        };

        let notif_tx = command_tx.clone();

        // Set notification handler
        let hub_addr = hub.addr.clone();
        peripheral.on_notification(Box::new(move |msg| {
            if let Ok(msg) = NotificationMessage::parse(&msg.value) {
                let notif = HubManagerMessage::Notification(hub_addr, msg);
                notif_tx.send(notif).unwrap();
            } else {
                error!("Message parse error: {:?}", msg);
            }
        }));

        // get LPF2 characteristic and subscribe to it
        let lpf_char = chars
            .iter()
            .find(|c| c.uuid == *blecharacteristic::LPF2_ALL)
            .context("Device does not advertise LPF2_ALL characteristic")?
            .clone();
        peripheral.subscribe(&lpf_char)?;

        let h = Box::new(match hub_type {
            HubType::TechnicMediumHub => {
                hubs::TechnicHub::init(peripheral, chars)?
            }
            _ => unimplemented!(),
        });
        hubs.insert(hub.addr, h);
        let controller = HubController {
            addr: hub.addr,
            hub_type,
            name,
            hub_manager_tx: command_tx,
        };
        Ok(controller)
    }

    fn disconnect(
        addr: BDAddr,
        hubs: &mut HashMap<BDAddr, Box<dyn Hub + Send + Sync>>,
    ) -> Result<()> {
        let hub = hubs.remove(&addr).context("Hub not registered")?;
        hub.disconnect()?;
        Ok(())
    }
}

/// Trait describing a generic hub.
pub trait Hub {
    fn name(&self) -> String;
    fn disconnect(&self) -> Result<()>;
    fn is_connected(&self) -> bool;
    // The init function cannot be a trait method until we have GAT :(
    //fn init(peripheral: P);
    fn properties(&self) -> &hubs::HubProperties;

    fn port_map(&self) -> &hubs::PortMap {
        &self.properties().port_map
    }

    // cannot provide a default implementation without access to the
    // Peripheral trait from here
    fn send_raw(&self, msg: &[u8]) -> Result<()>;

    fn send(&self, msg: NotificationMessage) -> Result<()>;

    fn subscribe(&self, char: Characteristic) -> Result<()>;

    /// Ideally the vec should be sorted somehow
    fn attached_io(&self) -> Vec<ConnectedIo>;

    fn process_io_event(&mut self, _evt: AttachedIo);
}

pub(crate) trait IdentifyHub {
    fn identify(&self) -> Option<HubType>;
}

/*
PeripheralProperties
{
 address: 90:84:2B:60:3C:B8,
 address_type: Public,
 local_name: Some("game"),
 tx_power_level: Some(-66),
 manufacturer_data: {919: [0, 128, 6, 0, 97, 0]},
 service_data: {},
 services: [00001623-1212-efde-1623-785feabcd123],
 discovery_count: 1,
 has_scan_response: false
}
*/
impl<P: Peripheral> IdentifyHub for P {
    fn identify(&self) -> Option<HubType> {
        use HubType::*;

        let props = self.properties();
        trace!("props:\n{:?}", props);

        if props
            .services
            .contains(&consts::bleservice::WEDO2_SMART_HUB)
        {
            return Some(Wedo2SmartHub);
        } else if props.services.contains(&consts::bleservice::LPF2_HUB) {
            if let Some(manufacturer_id) = props.manufacturer_data.get(&919) {
                // Can't do it with a match because some devices are just manufacturer
                // data while some use other characteristics
                if let Some(m) =
                    BLEManufacturerData::from_u8(manufacturer_id[1])
                {
                    use BLEManufacturerData::*;
                    return Some(match m {
                        DuploTrainBaseId => DuploTrainBase,
                        HubId => Hub,
                        MarioId => Mario,
                        MoveHubId => MoveHub,
                        RemoteControlId => RemoteControl,
                        TechnicMediumHubId => TechnicMediumHub,
                    });
                }
            }
        }
        None
    }
}
