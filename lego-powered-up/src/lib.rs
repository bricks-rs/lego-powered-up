use anyhow::{bail, Context, Result};
pub use btleplug::api::Peripheral;
use btleplug::api::{BDAddr, Characteristic};
use btleplug::api::{Central, CentralEvent};
use num_traits::FromPrimitive;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::{self, Duration};

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
use notifications::NotificationMessage;

#[allow(unused)]
mod consts;

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
    println!("  {}: Adapter {}");
    Ok(())
}

pub struct PoweredUp {
    manager: Manager,
    adapter: Arc<RwLock<Adapter>>,
    control_tx: Option<Sender<PoweredUpInternalControlMessage>>,
    pub hubs: Vec<Box<dyn Hub>>,
}

impl PoweredUp {
    pub fn devices() -> Result<Vec<Adapter>> {
        let manager = Manager::new()?;
        Ok(manager.adapters()?)
    }

    pub fn init() -> Result<Self> {
        Self::with_device(0)
    }

    pub fn with_device(dev: usize) -> Result<Self> {
        let manager = Manager::new()?;
        let adapters = manager.adapters()?;
        let adapter =
            adapters.into_iter().nth(dev).context("No adapter found")?;

        let mut pu = Self {
            manager,
            adapter: Arc::new(RwLock::new(adapter)),
            control_tx: None,
            hubs: Vec::new(),
        };
        pu.run()?;

        Ok(pu)
    }

    fn run(&mut self) -> Result<()> {
        let event_rx = self
            .adapter
            .write()
            .unwrap()
            .event_receiver()
            .context("Unable to access event receiver")?;
        let mut worker = PoweredUpInternal::new(self.adapter.clone());

        let (control_tx, control_rx) = channel(10);

        tokio::spawn(async move {
            worker.run(control_rx, event_rx).await.unwrap();
        });

        self.control_tx = Some(control_tx);

        self.adapter.write().unwrap().start_scan()?;

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = &self.control_tx {
            tx.send(PoweredUpInternalControlMessage::Stop).await?;
        }
        Ok(())
    }

    pub fn peripheral(&self, dev: BDAddr) -> Option<impl Peripheral> {
        self.adapter.write().unwrap().peripheral(dev)
    }

    pub fn create_hub(
        &self,
        hub_type: HubType,
        dev: BDAddr,
    ) -> Result<Box<dyn Hub>> {
        let peripheral = self
            .adapter
            .write()
            .unwrap()
            .peripheral(dev)
            .context("Unable to identify device")?;
        peripheral.connect()?;
        let chars = peripheral.discover_characteristics()?;

        let (notif_tx, notif_rx) = mpsc::channel();

        // Set notification handler
        peripheral.on_notification(Box::new(move |msg| {
            if let Ok(msg) = NotificationMessage::parse(&msg.value) {
                notif_tx.send(msg).unwrap();
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

        Ok(Box::new(match hub_type {
            HubType::TechnicMediumHub => {
                hubs::TechnicHub::init(peripheral, chars, notif_rx)?
            }
            _ => unimplemented!(),
        }))
    }

    pub async fn wait_for_hub(&self) -> Result<DiscoveredHub> {
        let timeout = Duration::from_secs(9999);
        self.wait_for_hub_filter_timeout_internal(None, timeout)
            .await
    }

    pub async fn wait_for_hub_filter(
        &self,
        filter: HubFilter,
    ) -> Result<DiscoveredHub> {
        let timeout = Duration::from_secs(9999);
        self.wait_for_hub_filter_timeout_internal(Some(filter), timeout)
            .await
    }

    pub async fn wait_for_hub_filter_timeout(
        &self,
        filter: HubFilter,
        timeout: Duration,
    ) -> Result<DiscoveredHub> {
        self.wait_for_hub_filter_timeout_internal(Some(filter), timeout)
            .await
    }

    async fn wait_for_hub_filter_timeout_internal(
        &self,
        filter: Option<HubFilter>,
        timeout: Duration,
    ) -> Result<DiscoveredHub> {
        let sleep = time::sleep(timeout);

        let (tx, rx) = oneshot::channel();
        let params = HubNotificationParams {
            response: tx,
            filter,
        };

        self.control_tx
            .as_ref()
            .unwrap()
            .send(PoweredUpInternalControlMessage::WaitForHub(params))
            .await?;

        tokio::select! {
            _ = sleep => {
                bail!("Timeout reached")
            }
            Ok(msg) = rx => {
               Ok(msg)
            }
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum DeviceNotificationMessage {
    HubDiscovered(DiscoveredHub),
}

#[derive(Debug)]
pub enum HubFilter {
    Name(String),
    Addr(String),
}

impl HubFilter {
    pub fn matches(&self, hub: &DiscoveredHub) -> bool {
        use HubFilter::*;
        match self {
            Name(n) => hub.name == *n,
            Addr(a) => hub.addr.to_string() == *a,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiscoveredHub {
    pub hub_type: HubType,
    pub addr: BDAddr,
    pub name: String,
}

#[derive(Debug)]
enum PoweredUpInternalControlMessage {
    Stop,
    WaitForHub(HubNotificationParams),
}

#[derive(Debug)]
struct HubNotificationParams {
    response: oneshot::Sender<DiscoveredHub>,
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
    pub async fn run(
        &mut self,
        mut control_channel: Receiver<PoweredUpInternalControlMessage>,
        event_rx: mpsc::Receiver<CentralEvent>,
    ) -> Result<()> {
        use DeviceNotificationMessage::*;
        info!("Starting PoweredUp connection manager");

        let (device_notification_sender, mut device_notification_receiver) =
            channel(16);
        let adapter_clone = self.adapter.clone();
        tokio::spawn(async move {
            PoweredUpInternal::btle_notification_listener(
                event_rx,
                device_notification_sender,
                adapter_clone,
            ).await
        });
        loop {
            tokio::select!(
                Some(msg) = device_notification_receiver.recv() => {
                    println!("PU INTERNAL MSG: {:?}", msg);
                    match msg {
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
                Some(msg) = control_channel.recv() => {
                    use PoweredUpInternalControlMessage::*;
                    match msg { // TODO disconnect all hubs
                        Stop => return Ok(()),
                        WaitForHub(params) => {
                            self.hub_notifications = Some(params);
                        }
                    }
                }
            );
        }
    }

    async fn btle_notification_listener(
        event_rx: mpsc::Receiver<CentralEvent>,
        device_notification_sender: Sender<DeviceNotificationMessage>,
        adapter: Arc<RwLock<Adapter>>,
    ) -> ! {
        use CentralEvent::*;
        info!("Starting btleplug async notification proxy");
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
                    .await
                    .expect("Device notification channel failed");
            }
        }
    }
}

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
    fn send(
        &self,
        port: Port,
        mode: u8,
        msg: &[u8],
        request_reply: bool,
    ) -> Result<()>;

    fn subscribe(&self, char: Characteristic) -> Result<()>;

    fn poll(&self) -> Option<NotificationMessage>;
}

pub trait IdentifyHub {
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
