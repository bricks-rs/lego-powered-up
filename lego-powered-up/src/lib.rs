// #![feature(exclusive_range_pattern)]
// #![allow(unused)]

pub use btleplug;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, PeripheralProperties,
    ScanFilter, ValueNotification,
};
use btleplug::platform::{Adapter, Manager, PeripheralId};

// std
use core::time::Duration;
pub use futures;
use futures::{stream::StreamExt, Stream};
use hubs::HubNotification;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
#[macro_use]
extern crate log;

// nostd
use core::fmt::Debug;
use core::pin::Pin;
use num_traits::FromPrimitive;

// Crate
pub mod consts;
// pub mod devices;
pub mod error;
pub mod hubs;
pub mod iodevice;
pub mod notifications;
pub mod setup;
mod tests;

pub use crate::consts::IoTypeId;
pub use crate::iodevice::IoDevice;
pub use hubs::Hub;

use consts::{BLEManufacturerData, HubType};
use notifications::{
    NetworkCommand, PortValueCombinedFormat, PortValueSingleFormat,
};

pub use error::{Error, OptionContext, Result};
// pub use consts::IoTypeId;

pub type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type NotificationStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

pub struct PoweredUp {
    adapter: Adapter,
}

impl PoweredUp {
    pub async fn adapters() -> Result<Vec<Adapter>> {
        let manager = Manager::new().await?;
        Ok(manager.adapters().await?)
    }

    pub async fn init() -> Result<Self> {
        let manager = Manager::new().await?;
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .next()
            .context("No adapter found")?;
        Self::with_adapter(adapter).await
    }

    pub async fn with_device_index(index: usize) -> Result<Self> {
        let manager = Manager::new().await?;
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .nth(index)
            .context("No adapter found")?;
        Self::with_adapter(adapter).await
    }

    pub async fn with_adapter(adapter: Adapter) -> Result<Self> {
        Ok(Self { adapter })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.adapter.start_scan(ScanFilter::default()).await?;
        Ok(())
    }

    pub async fn find_hub(&mut self) -> Result<Option<DiscoveredHub>> {
        let hubs = self.list_discovered_hubs().await?;
        Ok(hubs.into_iter().next())
    }

    pub async fn list_discovered_hubs(&mut self) -> Result<Vec<DiscoveredHub>> {
        let peripherals = self.adapter.peripherals().await?;
        let mut hubs = Vec::new();
        for peripheral in peripherals {
            let Some(props) = peripheral.properties().await? else{continue;};
            if let Some(hub_type) = identify_hub(&props).await? {
                hubs.push(DiscoveredHub {
                    hub_type,
                    addr: peripheral.id(),
                    name: props
                        .local_name
                        .unwrap_or_else(|| "unknown".to_string()),
                });
            }
        }
        Ok(hubs)
    }

    pub async fn wait_for_hub(&mut self) -> Result<DiscoveredHub> {
        self.wait_for_hub_filter(HubFilter::Null).await
    }

    pub async fn wait_for_hub_filter(
        &mut self,
        filter: HubFilter,
    ) -> Result<DiscoveredHub> {
        let mut events = self.adapter.events().await?;
        self.adapter.start_scan(ScanFilter::default()).await?;
        while let Some(event) = events.next().await {
            let CentralEvent::DeviceDiscovered(id) = event else { continue };
            // get peripheral info
            let peripheral = self.adapter.peripheral(&id).await?;
            // println!("{:?}", peripheral.properties().await?);
            let Some(props) = peripheral.properties().await? else { continue };
            if let Some(hub_type) = identify_hub(&props).await? {
                let hub = DiscoveredHub {
                    hub_type,
                    addr: id,
                    name: props
                        .local_name
                        .unwrap_or_else(|| "unknown".to_string()),
                };
                if filter.matches(&hub) {
                    self.adapter.stop_scan().await?;
                    return Ok(hub);
                }
            }
        }
        panic!()
    }

    pub async fn wait_for_hubs_filter(
        &mut self,
        filter: HubFilter,
        count: &u8,
    ) -> Result<Vec<DiscoveredHub>> {
        let mut events = self.adapter.events().await?;
        let mut hubs = Vec::new();
        self.adapter.start_scan(ScanFilter::default()).await?;
        while let Some(event) = events.next().await {
            let CentralEvent::DeviceDiscovered(id) = event else { continue };
            // get peripheral info
            let peripheral = self.adapter.peripheral(&id).await?;
            // println!("{:?}", peripheral.properties().await?);
            let Some(props) = peripheral.properties().await? else { continue };
            if let Some(hub_type) = identify_hub(&props).await? {
                let hub = DiscoveredHub {
                    hub_type,
                    addr: id,
                    name: props
                        .local_name
                        .unwrap_or_else(|| "unknown".to_string()),
                };
                if filter.matches(&hub) {
                    hubs.push(hub);
                }
                if hubs.len() == *count as usize {
                    self.adapter.stop_scan().await?;
                    return Ok(hubs);
                }
            }
        }
        panic!()
    }

    pub async fn create_hub(
        &mut self,
        hub: &DiscoveredHub,
    ) -> Result<Box<dyn Hub>> {
        info!("Connecting to hub {}...", hub.addr,);

        let peripheral = self.adapter.peripheral(&hub.addr).await?;
        peripheral.connect().await?;
        peripheral.discover_services().await?;
        // tokio::time::sleep(Duration::from_secs(2)).await;
        let chars = peripheral.characteristics();

        // dbg!(&chars);

        let lpf_char = chars
            .iter()
            .find(|c| c.uuid == *consts::blecharacteristic::LPF2_ALL)
            .context("Device does not advertise LPF2_ALL characteristic")?
            .clone();

        match hub.hub_type {
            // These have had some real life-testing.
            HubType::TechnicMediumHub
            | HubType::MoveHub
            | HubType::RemoteControl => Ok(Box::new(
                hubs::generic_hub::GenericHub::init(
                    peripheral,
                    lpf_char,
                    hub.hub_type,
                )
                .await?,
            )),
            // These are untested, but if they support the same "Lego Wireless protocol 3.0"
            // then they should probably work?
            HubType::Wedo2SmartHub
            | HubType::Hub
            | HubType::DuploTrainBase
            | HubType::Mario => Ok(Box::new(
                hubs::generic_hub::GenericHub::init(
                    peripheral,
                    lpf_char,
                    hub.hub_type,
                )
                .await?,
            )),
            // Here is some hub that advertises LPF2_ALL but is not in the known list.
            // Set kind to Unknown and give it a try, why not?
            _ => Ok(Box::new(
                hubs::generic_hub::GenericHub::init(
                    peripheral,
                    lpf_char,
                    HubType::Unknown,
                )
                .await?,
            )),
        }
    }

    pub async fn scan(
        &mut self,
    ) -> Result<impl Stream<Item = DiscoveredHub> + '_> {
        let events = self.adapter.events().await?;
        self.adapter.start_scan(ScanFilter::default()).await?;
        Ok(events.filter_map(|event| async {
            let CentralEvent::DeviceDiscovered(id) = event else { None? };
            // get peripheral info
            let peripheral = self.adapter.peripheral(&id).await.ok()?;
            println!("{:?}", peripheral.properties().await.unwrap());
            let Some(props) = peripheral.properties().await.ok()? else { None? };
            if let Some(hub_type) = identify_hub(&props).await.ok()? {
                let hub = DiscoveredHub {
                    hub_type,
                    addr: id,
                    name: props
                        .local_name
                        .unwrap_or_else(|| "unknown".to_string()),
                };
                Some(hub)
            } else { None }
        }))
    }

    pub async fn scan2(
        &mut self,
    ) -> Result<Pin<Box<dyn Stream<Item = DiscoveredHub> + Send + '_>>> {
        let events = self.adapter.events().await?;
        self.adapter.start_scan(ScanFilter::default()).await?;
        Ok(Box::pin(events.filter_map(|event| async {
            let CentralEvent::DeviceDiscovered(id) = event else { None? };
            // get peripheral info
            let peripheral = self.adapter.peripheral(&id).await.ok()?;
            println!("{:?}", peripheral.properties().await.unwrap());
            let Some(props) = peripheral.properties().await.ok()? else { None? };
            if let Some(hub_type) = identify_hub(&props).await.ok()? {
                let hub = DiscoveredHub {
                    hub_type,
                    addr: id,
                    name: props
                        .local_name
                        .unwrap_or_else(|| "unknown".to_string()),
                };
                Some(hub)
            } else { None }
        })))
    }
}

/// Properties by which to filter discovered hubs
#[derive(Debug)]
pub enum HubFilter {
    /// Hub name must match the provided value
    Name(String),
    /// Hub address must match the provided value
    Addr(String),
    /// Always matches
    Null,
}

impl HubFilter {
    /// Test whether the discovered hub matches the provided filter mode
    pub fn matches(&self, hub: &DiscoveredHub) -> bool {
        use HubFilter::*;
        match self {
            Name(n) => hub.name == *n,
            Addr(a) => format!("{:?}", hub.addr) == *a,
            Null => true,
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
    pub addr: PeripheralId,
    /// Friendly name of the hub, as set in the PoweredUp/Control+ apps
    pub name: String,
}

async fn identify_hub(props: &PeripheralProperties) -> Result<Option<HubType>> {
    use HubType::*;

    if props
        .services
        .contains(&consts::bleservice::WEDO2_SMART_HUB)
    {
        return Ok(Some(Wedo2SmartHub));
    } else if props.services.contains(&consts::bleservice::LPF2_HUB) {
        if let Some(manufacturer_id) = props.manufacturer_data.get(&919) {
            // Can't do it with a match because some devices are just manufacturer
            // data while some use other characteristics
            if let Some(m) = BLEManufacturerData::from_u8(manufacturer_id[1]) {
                use BLEManufacturerData::*;
                return Ok(Some(match m {
                    DuploTrainBaseId => DuploTrainBase,
                    HubId => Hub,
                    MarioId => Mario,
                    MoveHubId => MoveHub,
                    RemoteControlId => RemoteControl,
                    TechnicMediumHubId => TechnicMediumHub,
                }));
            }
        }
    }
    Ok(None)
}

pub struct ConnectedHub {
    pub name: String,
    pub mutex: HubMutex,
    pub kind: HubType,
}
impl ConnectedHub {
    pub async fn setup_hub(created_hub: Box<dyn Hub>) -> Result<ConnectedHub> {
        let connected_hub = ConnectedHub {
            kind: created_hub.kind(),
            name: created_hub.name().await?,
            mutex: Arc::new(Mutex::new(created_hub)),
        };
        // Create forwarding channels and store in hub so we can create receivers on demand
        {
            let lock = &mut connected_hub.mutex.lock().await;
            lock.channels().singlevalue_sender =
                Some(broadcast::channel::<PortValueSingleFormat>(16).0);
            lock.channels().combinedvalue_sender =
                Some(broadcast::channel::<PortValueCombinedFormat>(16).0);
            lock.channels().networkcmd_sender =
                Some(broadcast::channel::<NetworkCommand>(16).0);
            lock.channels().hubnotification_sender =
                Some(broadcast::channel::<HubNotification>(16).0);
        }

        // Set up notification handler
        let hub_mutex = connected_hub.mutex.clone();
        {
            let lock = &mut connected_hub.mutex.lock().await;
            let stream: NotificationStream =
                lock.peripheral().notifications().await?;
            let senders = (
                lock.channels().singlevalue_sender.as_ref().unwrap().clone(),
                lock.channels()
                    .combinedvalue_sender
                    .as_ref()
                    .unwrap()
                    .clone(),
                lock.channels().networkcmd_sender.as_ref().unwrap().clone(),
                lock.channels()
                    .hubnotification_sender
                    .as_ref()
                    .unwrap()
                    .clone(),
            );
            tokio::spawn(async move {
                crate::hubs::io_event::io_event_handler(
                    stream, hub_mutex, senders,
                )
                .await
                .expect("Error setting up main notification handler");
            });
        }

        // Subscribe to btleplug peripheral
        {
            let lock = connected_hub.mutex.lock().await;
            match lock.peripheral().subscribe(&lock.characteristic()).await {
                Ok(()) => (),
                // We got a peri connection but can't subscribe. Can happen if the hub has almost timed out
                // waiting for a connection; it seemingly connects but then turns off. On Windows the error
                // returned was a HRESULT: Operation aborted
                Err(e) => {
                    eprintln!(
                        "Error subscribing to peripheral notifications: {:#?}",
                        e
                    )
                }
            }
        }
        // Wait for devices to be collected. This is set to a very long time because notifications
        // from the hub sometimes lag, and we don't know how many devices to expect.
        tokio::time::sleep(Duration::from_millis(3000)).await;

        Ok(connected_hub)
    }
}
