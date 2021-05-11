use anyhow::{Context, Result};
pub use btleplug::api::Peripheral;
use btleplug::api::{BDAddr, PeripheralProperties};
use btleplug::api::{Central, CentralEvent};
use num_traits::FromPrimitive;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, manager::Manager};

#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};

#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};

#[allow(unused)]
use log::{debug, error, info, trace, warn};

use consts::*;
mod consts;

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

fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().unwrap();
    adapters.into_iter().next().unwrap()
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum PoweredUpEvent {
    HubDiscovered(HubType, BDAddr),
}

pub struct PoweredUp {
    manager: Manager,
    adapter: Arc<RwLock<Adapter>>,
    event_rx: Option<Receiver<CentralEvent>>,
    pu_event_tx: Option<Sender<PoweredUpEvent>>,
    pu_event_rx: Option<Receiver<PoweredUpEvent>>,
    worker_thread: Option<JoinHandle<Result<()>>>,
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
        let event_rx = Some(
            adapter
                .event_receiver()
                .context("Unable to access event receiver")?,
        );

        let (pu_event_tx, pu_event_rx) = mpsc::channel();

        Ok(Self {
            manager,
            adapter: Arc::new(RwLock::new(adapter)),
            event_rx,
            pu_event_tx: Some(pu_event_tx),
            pu_event_rx: Some(pu_event_rx),
            worker_thread: None,
            hubs: Vec::new(),
        })
    }

    pub fn event_receiver(&mut self) -> Option<Receiver<PoweredUpEvent>> {
        self.pu_event_rx.take()
    }

    pub fn start_scan(&mut self) -> Result<()> {
        self.adapter.write().unwrap().start_scan()?;

        let mut worker = Worker {
            pu_event_tx: self
                .pu_event_tx
                .take()
                .context("Unable to access event transmitter")?,
            event_rx: self
                .event_rx
                .take()
                .context("Unable to access btle event receiver")?,
            adapter: self.adapter.clone(),
        };

        let handle = thread::spawn(move || worker.run());
        self.worker_thread = Some(handle);
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
        let peripheral =
            self.adapter.write().unwrap().peripheral(dev).context("Unable to identify device")?;
        peripheral.connect()?;

            Ok(Box::new(match hub_type {
                HubType::TechnicMediumHub => {
                    TechnicHub {
                        peripheral
                    }
                }
                _ => unimplemented!(),
            }))
    }

    pub fn connect(&self, dev: BDAddr) -> Result<Box<dyn Hub>> {
        let peripheral = self
            .adapter
            .write()
            .unwrap()
            .peripheral(dev)
            .context("No device found")?;

        peripheral.connect()?;

        let chars = peripheral.discover_characteristics();
        if peripheral.is_connected() {
            println!(
                "Discover peripheral : \'{:?}\' characteristics...",
                peripheral.properties().local_name
            );
            for chars_vector in chars.into_iter() {
                for char_item in chars_vector.iter() {
                    println!("{:?}", char_item);
                }
            }
            println!(
                "disconnecting from peripheral : {:?}...",
                peripheral.properties().local_name
            );
            peripheral
                .disconnect()
                .expect("Error on disconnecting from BLE peripheral ");
        }
        todo!()
    }
}

struct Worker {
    pub pu_event_tx: Sender<PoweredUpEvent>,
    pub event_rx: Receiver<CentralEvent>,
    pub adapter: Arc<RwLock<Adapter>>,
}

impl Worker {
    pub fn run(&mut self) -> Result<()> {
        use CentralEvent::*;
        loop {
            // This is in a loop rather than a while let so that the
            // mpsc error gets propagated
            let evt = self.event_rx.recv()?;
            match evt {
                DeviceDiscovered(dev) => {
                    let adapter = self.adapter.write().unwrap();
                    let peripheral = adapter.peripheral(dev).unwrap();
                    debug!(
                        "peripheral : {:?} is connected: {:?}",
                        peripheral.properties().local_name,
                        peripheral.is_connected()
                    );
                    if peripheral.properties().local_name.is_some()
                        && !peripheral.is_connected()
                    {
                        if let Some(hub_type) = peripheral.identify() {
                            debug!("Looks like a '{:?}' hub!", hub_type);
                            self.pu_event_tx.send(
                                PoweredUpEvent::HubDiscovered(hub_type, dev),
                            )?;
                        } else {
                            debug!("Device does not look like a PoweredUp Hub");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let manager = Manager::new().unwrap();
    let adapter = get_central(&manager);
    println!("connecting to BLE adapter: ...");

    print_adapter_info(0, &adapter).unwrap();
    let event_rx = adapter
        .event_receiver()
        .expect("Can't scan BLE adapter for connected devices...");

    // start the scan
    adapter.start_scan().unwrap();

    while let Ok(evt) = event_rx.recv() {
        // all peripheral devices in range
        //for peripheral in adapter.peripherals().iter() {
        println!("Event: {:?}", evt);
        if let CentralEvent::DeviceDiscovered(dev) = evt {
            let peripheral = adapter.peripheral(dev).unwrap();
            println!(
                "peripheral : {:?} is connected: {:?}",
                peripheral.properties().local_name,
                peripheral.is_connected()
            );
            if peripheral.properties().local_name.is_some()
                && !peripheral.is_connected()
            {
                if let Some(hub_type) = peripheral.identify() {
                    println!("Looks like a '{:?}' hub!", hub_type);
                } else {
                    println!("Device does not look like a PoweredUp Hub");
                }

                //let hub = register_hub(&peripheral.properties());
                /*println!(
                    "start connect to peripheral : {:?}...",
                    peripheral.properties().local_name
                );
                peripheral
                    .connect()
                    .expect("Can't connect to peripheral...");
                println!(
                    "now connected (\'{:?}\') to peripheral : {:?}...",
                    peripheral.is_connected(),
                    peripheral.properties().local_name
                );
                let chars = peripheral.discover_characteristics();
                if peripheral.is_connected() {
                    println!(
                        "Discover peripheral : \'{:?}\' characteristics...",
                        peripheral.properties().local_name
                    );
                    for chars_vector in chars.into_iter() {
                        for char_item in chars_vector.iter() {
                            println!("{:?}", char_item);
                        }
                    }
                    println!(
                        "disconnecting from peripheral : {:?}...",
                        peripheral.properties().local_name
                    );
                    peripheral.disconnect().expect(
                        "Error on disconnecting from BLE peripheral ",
                    );
                }*/
                eprintln!(
                    "Not connecting because don't want to interrupt denon"
                );
            } else {
                //sometimes peripheral is not discovered completely
                eprintln!(
                    "SKIP connect to UNKNOWN peripheral : {:?}",
                    peripheral
                );
            }
        }
    }
}

/*pub fn register_hub(peripheral: &PeripheralProperties) -> Option<impl Hub> {
    Option::<TechnicHub<P>>::None
}*/

pub trait Hub {
    fn name(&self) -> String;
    fn disconnect(&self) -> Result<()>;
    fn is_connected(&self) -> bool;
}



pub struct TechnicHub<P: Peripheral> {
    peripheral: P,
}

impl <P: Peripheral>Hub for TechnicHub<P> {
    fn name(&self) -> String {
        self.peripheral.properties().local_name.unwrap_or_default()
    }

    fn disconnect(&self) -> Result<()> {
        if self.is_connected() {
            self.peripheral.disconnect()?;
        }
        Ok(())
    }

     fn is_connected(&self) -> bool {
        self.peripheral.is_connected()
    }
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
