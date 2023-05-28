// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Specific implementations for each of the supported hubs.

#![allow(unused)]

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use btleplug::api::ValueNotification;

use std::collections::{HashMap, BTreeMap};
use std::fmt::Debug;

use crate::{PoweredUp, HubFilter, devices::Device, error::Error};
use crate::notifications::NotificationMessage;
use crate::notifications::NetworkCommand::ConnectionRequest;
use crate::notifications::*;
use crate::consts::*;
use devices::iodevice::IoDevice;



use crate::devices::{self, };
use crate::error::{OptionContext, Result};
use crate::notifications::{ModeInformationRequest, ModeInformationType,
     InformationRequest, InformationType,  
     HubAction, HubActionRequest, InputSetupSingle, PortModeInformationType};
use devices::IoTypeId;
use devices::iodevice::*;

use futures::stream::{StreamExt, FuturesUnordered, Stream};
use futures::{future, select};
use core::pin::Pin;
use std::sync::{Arc};
use tokio::sync::Mutex;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

/// Trait describing a generic hub.
#[async_trait::async_trait]
pub trait Hub: Debug + Send + Sync {
    // fn cached_name(&self) -> String;

    async fn name(&self) -> Result<String>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> Result<bool>;
    // The init function cannot be a trait method until we have GAT :(
    //fn init(peripheral: P);
    fn properties(&self) -> &HubProperties;
    fn peripheral(&self) -> &Peripheral;
    fn characteristic(&self) -> &Characteristic;
    fn connected_io(&mut self) -> &mut BTreeMap<u8, IoDevice>;

    // Port information
    async fn request_port_info(&self, port_id: u8, infotype: InformationType) -> Result<()> {
        let msg =
        NotificationMessage::PortInformationRequest(InformationRequest {
            port_id,
            information_type: infotype,
        });
        self.send(msg).await
    }
    async fn req_mode_info(&self, port_id: u8, mode: u8, infotype: ModeInformationType) -> Result<()> {
        let msg =
        NotificationMessage::PortModeInformationRequest(ModeInformationRequest {
            port_id,
            mode,
            information_type: infotype,
        });
        self.send(msg).await
    }

    async fn set_port_mode(&self, port_id: u8, mode: u8, delta: u32, notification_enabled: bool) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id,
                mode, 
                delta,
                notification_enabled,
            });
        self.send(mode_set_msg).await
    }

    async fn hub_action(&self, action_type: HubAction) -> Result<()> {
        let msg =
        NotificationMessage::HubActions(HubActionRequest {
            action_type,
        });
        self.send(msg).await
    }
   
    async fn send(&self, msg: NotificationMessage) -> Result<()> {
        let buf = msg.serialise();
        self.peripheral()
            .write(self.characteristic(), &buf, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }

    fn attach_io(&mut self, device_to_insert: IoDevice) -> Result<()>;

    // async fn port_map(&self) -> &PortMap {
    //     &self.properties().await.port_map
    // }

    // cannot provide a default implementation without access to the
    // Peripheral trait from here
    async fn send_raw(&self, msg: &[u8]) -> Result<()>;

    // fn send(&self, msg: NotificationMessage) -> Result<()>;

    async fn subscribe(&self, char: Characteristic) -> Result<()>;

    /// Ideally the vec should be sorted somehow
    // async fn attached_io(&self) -> Vec<IoDevice>;

    // fn process_io_event(&mut self, _evt: AttachedIo);

    async fn port(&self, port_id: Port) -> Result<Box<dyn Device>>;
    async fn enable_from_port(&self, port_id: u8) -> Result<Box<dyn Device>>;
    // async fn enable_from_kind(&self, port_id: u8) -> Result<Box<dyn Device>>;
}

pub type VersionNumber = u8;

/// Propeties of a hub
#[derive(Debug, Default)]
pub struct HubProperties {
    /// Friendly name, set via the PoweredUp or Control+ apps
    pub name: String,
    /// Firmware revision
    pub fw_version: String,
    /// Hardware revision
    pub hw_version: String,
    /// BLE MAC address
    pub mac_address: String,
    /// Battery level
    pub battery_level: usize,
    /// BLE signal strength
    pub rssi: i16,
    /// Mapping from port type to port ID. Internally (to the hub) each
    /// port has a hardcoded identifier
    pub port_map: PortMap,
}

pub type PortMap = HashMap<Port, u8>;

/// Ports supported by any hub
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Port {
    /// Motor A
    A,
    /// Motor B
    B,
    /// Motor C
    C,
    /// Motor D
    D,
    AB, // Move Hub
    HubLed,
    CurrentSensor,
    VoltageSensor,
    Accelerometer,
    GyroSensor,
    TiltSensor,
    GestureSensor,
    TemperatureSensor1,
    TemperatureSensor2,
    InternalMotor,
    Rssi,
    RemoteA,
    RemoteB,
    Virtual(u8),
}

impl Port {
    /// Returns the ID of the port
    pub fn id(&self) -> u8 {
        todo!()
    }
}

/// Struct representing a device connected to a port
// #[derive(Debug, Clone)]
// pub struct ConnectedIo {
//     /// Name/type of device
//     pub port: Port,
//     /// Internal numeric ID of the device
//     pub port_id: u8,
//     /// Device firmware revision
//     pub fw_rev: VersionNumber,
//     /// Device hardware revision
//     pub hw_rev: VersionNumber,
// }
#[derive(Debug, Clone)]
pub struct ConnectedIo {
    /// Name/type of device
    pub io_type_id: IoTypeId,
    /// Internal numeric ID of the device
    pub port_id: u8,
}


pub mod technic_hub;
pub mod remote;
pub mod move_hub;

pub async fn handle_notification_stream(mut stream: PinnedStream, mutex: HubMutex, hub_name: String) {
    while let Some(data) = stream.next().await {
        // println!("Received data from {:?} [{:?}]: {:?}", hub_name, data.uuid, data.value);

        let r = NotificationMessage::parse(&data.value);
        match r {
            Ok(n) => {
                // dbg!(&n);
                match n {
                    NotificationMessage::HubAttachedIo(io_event) => {
                        match io_event {
                            AttachedIo{port, event} => {
                                let port_id = port;
                                match event {
                                    IoAttachEvent::AttachedIo{io_type_id, hw_rev, fw_rev} => {
                                        let aio = IoDevice::new(io_type_id, port_id);
                                        {
                                            let mut hub = mutex.lock().await;
                                            hub.attach_io(aio);
                                            hub.request_port_info(port_id, InformationType::ModeInfo).await;
                                            hub.request_port_info(port_id, InformationType::PossibleModeCombinations).await;
                                        }
                                    }
                                    IoAttachEvent::DetachedIo{} => {}
                                    IoAttachEvent::AttachedVirtualIo {port_a, port_b }=> {}
                                }
                            }
                        }
                    }
                    NotificationMessage::PortInformation(val) => {
                        match val {
                            PortInformationValue{port_id, information_type} => {
                                let port_id = port_id;
                                match information_type {
                                    PortInformationType::ModeInfo{capabilities, mode_count, input_modes, output_modes} => {
                                        {
                                            let mut hub = mutex.lock().await;
                                            let mut port = hub.connected_io().get_mut(&port_id).unwrap();
                                            port.set_mode_count(mode_count);
                                            port.set_capabilities(capabilities.0);
                                            port.set_modes(input_modes, output_modes);
                                      
                                            // let count = 
                                            for mode_id in 0..mode_count {
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Name).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Raw).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Pct).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Si).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Symbol).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Mapping).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::MotorBias).await;
                                                // hub.request_mode_info(port_id, mode_id, ModeInformationType::CapabilityBits).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::ValueFormat).await;
                                            }
                                        }
                                    }
                                    PortInformationType::PossibleModeCombinations(combs) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_valid_combos(combs);   
                                    }
                                }
                            }
                        }
                    }
                    NotificationMessage::PortModeInformation(val ) => {
                        match val {
                            PortModeInformationValue{port_id, mode, information_type} => {
                                match information_type {
                                    PortModeInformationType::Name(name) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_name(mode, name);
                                    }
                                    PortModeInformationType::RawRange{min, max } => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_raw(mode, min, max);
                                    }
                                    PortModeInformationType::PctRange{min, max } => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_pct(mode, min, max);
                                    }
                                    PortModeInformationType::SiRange{min, max } => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_si(mode, min, max);
                                    }
                                    PortModeInformationType::Symbol(symbol) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_symbol(mode, symbol);
                                    }
                                    PortModeInformationType::Mapping{input, output} => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_mapping(mode, input, output);
                                    }
                                    PortModeInformationType::MotorBias(bias) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_motor_bias(mode, bias);
                                    }
                                    // PortModeInformationType::CapabilityBits(name) => {
                                    //     let mut hub = mutex.lock().await;
                                    //     hub.connected_io().get_mut(&port_id).unwrap().set_mode_cabability(mode, name);  //set_mode_capability not implemented
                                    // }
                                    PortModeInformationType::ValueFormat(format) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().set_mode_valueformat(mode, format);
                                    }
                                    _ => ()
                                }
                            }

                        }
                    }
                    NotificationMessage::HubProperties(val) => {}
                    NotificationMessage::HubActions(val) => {}
                    NotificationMessage::HubAlerts(val) => {}
                    NotificationMessage::GenericErrorMessages(val) => {}
                    NotificationMessage::HwNetworkCommands(val) => {}
                    NotificationMessage::FwLockStatus(val) => {}

                    NotificationMessage::PortValueSingle(val) => {}
                    NotificationMessage::PortValueCombinedmode(val) => {}
                    NotificationMessage::PortInputFormatSingle(val) => {}
                    NotificationMessage::PortInputFormatCombinedmode(val) => {}
                    NotificationMessage::PortOutputCommandFeedback(val ) => {}


                    _ => ()
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

    }  
}