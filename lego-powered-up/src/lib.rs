#![allow(unused)]
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, PeripheralProperties,
    ScanFilter, //ValueNotification
};
use btleplug::platform::{Adapter, Manager, PeripheralId, Peripheral};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};

use futures::{stream::StreamExt, Stream};
use num_traits::FromPrimitive;

#[macro_use]
extern crate log;

pub mod consts;
pub mod devices;
pub mod error;
pub mod hubs;
pub mod notifications;

pub use btleplug;
pub use error::{Error, OptionContext, Result};
pub use futures;

use consts::{BLEManufacturerData, HubType};
pub use hubs::Hub;

pub struct PoweredUp {
    adapter: Adapter,
    // peri: Peripheral,
    // characteristic: Characteristic
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

    pub async fn scan(&mut self) -> Result<impl Stream<Item = DiscoveredHub> + '_> {
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

    pub async fn wait_for_hub(&mut self) -> Result<DiscoveredHub> {
        self.wait_for_hub_filter(HubFilter::Null).await
    }

    pub async fn wait_for_hub_filter(&mut self, filter: HubFilter) -> Result<DiscoveredHub> {
        let mut events = self.adapter.events().await?;
        self.adapter.start_scan(ScanFilter::default()).await?;
        while let Some(event) = events.next().await {
            let CentralEvent::DeviceDiscovered(id) = event else { continue };
            // get peripheral info
            let peripheral = self.adapter.peripheral(&id).await?;
            println!("{:?}", peripheral.properties().await?);
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

    pub async fn wait_for_hubs_filter(&mut self, filter: HubFilter, count: &u8) -> Result<Vec<DiscoveredHub>> {
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


   
    pub async fn create_hub(&mut self, hub: &DiscoveredHub,) -> Result<Box<dyn Hub>> {
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
    
        // match hub.hub_type {
        //     HubType::TechnicMediumHub => {
        //         Ok(Box::new(hubs::technic_hub::TechnicHub::init(
        //             peripheral, lpf_char, hub.hub_type).await?))
        //     }
        //     HubType::RemoteControl => {
        //         Ok(Box::new(hubs::remote::RemoteControl::init(
        //             peripheral, lpf_char, HubType::RemoteControl).await?))
        //     }
        //     HubType::MoveHub => {
        //         Ok(Box::new(hubs::move_hub::MoveHub::init(
        //             peripheral, lpf_char, hub.hub_type).await?))
        //     }
        //     _ => unimplemented!("Hub type not implemented."),
        // }

        match hub.hub_type {
            // These have had some real life-testing.
            HubType::TechnicMediumHub |
            HubType::MoveHub |
            HubType::RemoteControl  => {
                Ok(Box::new(hubs::generic_hub::GenericHub::init(
                    peripheral, lpf_char, hub.hub_type).await?))
            }
            // These are untested, but if they support the same "Lego Wireless protocol 3.0"
            // then they should probably work with the generic implementation?
            HubType::Wedo2SmartHub |
            HubType::Hub |
            HubType::DuploTrainBase |
            HubType::Mario          => {
            Ok(Box::new(hubs::generic_hub::GenericHub::init(
                peripheral, lpf_char, hub.hub_type).await?))
            }
            // Here is some hub that advertises LPF2_ALL but is not in the known list.
            // Set kind to Unknown and give it a try, why not?
            _ => {
                Ok(Box::new(hubs::generic_hub::GenericHub::init(
                peripheral, lpf_char, HubType::Unknown).await?))
            }
        }
    }
}

use core::pin::Pin;
use std::sync::{Arc};    
use tokio::sync::Mutex;
use btleplug::api::ValueNotification;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;
pub struct ConnectedHub {
    pub name: String,
    pub mutex: HubMutex,
    // pub stream: PinnedStream
    pub kind: HubType,
}
impl ConnectedHub {
    pub async fn setup_hub (created_hub: Box<dyn Hub>) -> ConnectedHub {    // Return result later
        let connected_hub = ConnectedHub {
            // kind: created_hub.kind(),
            kind: created_hub.kind(),
            name: created_hub.name().await.unwrap(),                                                    // And here
            mutex: Arc::new(Mutex::new(created_hub)),
            // stream: created_hub.peripheral().notifications().await.unwrap()
        };
        
        
        
        // Set up handler
        let name_to_handler = connected_hub.name.clone();
        let mutex_to_handler = connected_hub.mutex.clone();
        let mutex_to_get_stream = connected_hub.mutex.clone();
        {
            let mut lock = mutex_to_get_stream.lock().await;
            let stream_to_handler: PinnedStream = lock.peripheral().notifications().await.unwrap();    // Can use ? here then
            tokio::spawn(async move {
                crate::hubs::io_event::io_event_handler(
                    // created_stream, mutex_handle, &name).await;
                    stream_to_handler, mutex_to_handler, name_to_handler).await;
            });
        }
        
        // Subscribe
        {
            let lock = connected_hub.mutex.lock().await;
            lock.peripheral().subscribe(&lock.characteristic()).await.unwrap();
        }

        connected_hub
    }

    pub fn create_channel() -> () {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<u8>(3);
    }

    pub async fn set_up_handler(mutex: HubMutex) -> (PinnedStream, HubMutex, String) {
        let mutex_to_handler = mutex.clone();
        let mut lock = mutex.lock().await;
        let name_to_handler = lock.name().await.unwrap();
        let stream_to_handler: PinnedStream = lock.peripheral().notifications().await.unwrap();     
    
        (stream_to_handler, mutex_to_handler, name_to_handler)
    } 

}
// pub struct HandlerSetup {
//     hub_name: String,
//     stream: PinnedStream,
//     mutex: HubMutex, 
//     tx: tokio::sync::broadcast::Sender<T>
// }

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
