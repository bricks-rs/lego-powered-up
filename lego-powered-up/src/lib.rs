#![allow(unused)]
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, PeripheralProperties,
    ScanFilter, //ValueNotification
};
use btleplug::platform::{Adapter, Manager, PeripheralId, Peripheral};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};

use async_trait::async_trait;
use devices::{IoTypeId, MessageType};
use devices::iodevice::{IoDevice, PortMode};
use devices::sensor::*;
use futures::{stream::StreamExt, Stream};
use hubs::io_event::ValWrap;
use notifications::{PortValueSingleFormat, ValueFormatType, PortValueSingleFormat2, PortValueCombinedFormat, NetworkCommand};
use num_traits::FromPrimitive;
use tokio::task::JoinHandle;

use std::collections::HashMap;
use std::fmt::Debug;
use core::pin::Pin;
use std::sync::{Arc};    
use tokio::sync::Mutex;
use btleplug::api::ValueNotification;

#[macro_use]
extern crate log;

pub mod consts;
pub mod devices;
pub mod error;
pub mod hubs;
pub mod notifications;
mod tests;

pub use btleplug;
pub use error::{Error, OptionContext, Result};
pub use futures;

use consts::{BLEManufacturerData, HubType};
pub use hubs::Hub;

use tokio::sync::broadcast::{self, Receiver};
use crate::notifications::NotificationMessage;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;
use crate::hubs::io_event::ChannelNotification;

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

        match hub.hub_type {
            // These have had some real life-testing.
            HubType::TechnicMediumHub |
            HubType::MoveHub |
            HubType::RemoteControl  => {
                Ok(Box::new(hubs::generic_hub::GenericHub::init(
                    peripheral, lpf_char, hub.hub_type).await?))
            }
            // These are untested, but if they support the same "Lego Wireless protocol 3.0"
            // then they should probably work?
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


pub struct ConnectedHub {
    pub name: String,
    pub mutex: HubMutex,
    // pub stream: PinnedStream
    pub kind: HubType,
    // pub msg_channels: HashMap<MessageType, broadcast::Sender<ChannelNotification>>, 
    // pub pvs_sender: Option<broadcast::Sender<PortValueSingleFormat>>
}
impl ConnectedHub {
    pub async fn setup_hub (created_hub: Box<dyn Hub>) -> ConnectedHub {    // Return result later
        let mut connected_hub = ConnectedHub {
            kind: created_hub.kind(),
            name: created_hub.name().await.unwrap(),                                                    // And here
            mutex: Arc::new(Mutex::new(created_hub)),
            // stream: created_hub.peripheral().notifications().await.unwrap()
            // msg_channels: HashMap::new(),
            // pvs_sender: None,
        };
        
        // Set up hub handlers
        //      Attached IO
        let name_to_handler = connected_hub.name.clone();
        let mutex_to_handler = connected_hub.mutex.clone();
        // let created_channels = ConnectedHub::create_channels();
        // connected_hub.msg_channels = created_channels.0;
        let singlevalue_sender = broadcast::channel::<PortValueSingleFormat>(3).0;
        let combinedvalue_sender = broadcast::channel::<PortValueCombinedFormat>(3).0;
        let networkcmd_sender = broadcast::channel::<NetworkCommand>(3).0;
        {
            let mut lock = &mut connected_hub.mutex.lock().await;
            let stream_to_handler: PinnedStream = lock.peripheral().notifications().await.unwrap();    // Can use ? here then
            lock.channels().singlevalue_sender = Some(singlevalue_sender.clone());
            lock.channels().combinedvalue_sender = Some(combinedvalue_sender.clone());
            lock.channels().networkcmd_sender = Some(networkcmd_sender.clone());
            tokio::spawn(async move {
                crate::hubs::io_event::io_event_handler(
                    stream_to_handler, 
                    mutex_to_handler,
          name_to_handler,
            singlevalue_sender,
                    combinedvalue_sender,
                    networkcmd_sender
                ).await;
            });
        }
        //  TODO    Hub alerts etc.
        
        // Subscribe to btleplug peripheral
        {
            let lock = connected_hub.mutex.lock().await;
            lock.peripheral().subscribe(&lock.characteristic()).await.unwrap();
        }

        connected_hub
    }
    



    // TODO: Return Result
    pub async fn set_up_handler(mutex: HubMutex) -> (PinnedStream, HubMutex, String) {
        let mutex_to_handler = mutex.clone();
        let mut lock = mutex.lock().await;
        let name_to_handler = lock.name().await.unwrap();
        let stream_to_handler: PinnedStream = lock.peripheral().notifications().await.unwrap();     
    
        (stream_to_handler, mutex_to_handler, name_to_handler)
    } 

    // pub async fn enable_8bit_sensor(&self, io_kind: IoTypeId, mode: u8, delta: u32, sub_channel: broadcast::Sender<PortValueSingleFormat>) -> Result<(JoinHandle<()>)> {

    // pub async fn sub_psv(&self, io_kind: IoTypeId, mode: u8, delta: u32, sub_channel: broadcast::Sender<PortValueSingleFormat>) -> Result<(JoinHandle<()>)> {
    //     let mut lock = self.mutex.lock().await;
    //     let mut device = lock.get_from_kind(io_kind).await?;
    //     // device.single_value_sensor_enable(mode, delta);

    //     // // get valueformat based on mode
    //     let mut vf_opt: Option<ValueFormatType> = None; 
    //     if let Some(portmode) = device.modes.get(&mode) {
    //         vf_opt = Some(portmode.value_format);
    //     }

    //     if let Some(pvs_sender) = &self.pvs_sender {
    //         match &self.pvs_sender {
    //             Some(tx) => {
    //                 let mut rx = tx.subscribe();
    //                 Ok(tokio::spawn(async move {
    //                     while let Ok(data) = rx.recv().await {
    //                         if data.port_id != device.port {
    //                             continue;
    //                         }
    //                         sub_channel.send(data);
    //                     }
    //                 }))
                        
    //             }
    //             None => Err(Error::NoneError(String::from("Unsupported msg type")))
    //         }   
    //     } else {
    //         Err(Error::NoneError(String::from("Unsupported msg type")))
    //     }
    // }


    

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

// async fn set

#[async_trait]
pub trait NotificationHandler: Debug + Send {  //+ Sync {

}


