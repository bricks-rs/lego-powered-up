
use core::pin::Pin;
use crate::futures::stream::{Stream, StreamExt};
use crate::btleplug::api::ValueNotification;

use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::broadcast;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

use crate::notifications::*;
use crate::devices::iodevice::*;

pub async fn io_event_handler(mut stream: PinnedStream, mutex: HubMutex, hub_name: String) {
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
                                        {
                                            let mut hub = mutex.lock().await;
                                            let p = hub.peripheral().clone();
                                            let c = hub.characteristic().clone();
                                            hub.attach_io(
                                                IoDevice::new_with_handles(
                                                    io_type_id, port_id, p, c));
                                            // hub.attach_io(
                                            //     IoDevice::new(
                                            //                 io_type_id, port_id));
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