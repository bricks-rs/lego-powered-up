#![allow(unused)]

use core::pin::Pin;
use futures::stream::{Stream, StreamExt};

use std::fmt::Debug;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::broadcast::{self, Sender};
use btleplug::api::ValueNotification;

use crate::IoDevice;
use crate::error::{Error, Result, OptionContext};
use crate::notifications::*;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;


pub async fn io_event_handler(mut stream: PinnedStream, mutex: HubMutex, 
                            senders: (Sender<PortValueSingleFormat>, 
                                      Sender<PortValueCombinedFormat>,
                                      Sender<NetworkCommand>)
                            ) -> Result<()> {
    const DIAGNOSTICS: bool = true;
    while let Some(data) = stream.next().await {
        // println!("Received data from {:?} [{:?}]: {:?}", hub_name, data.uuid, data.value);  // Dev use

        let r = NotificationMessage::parse(&data.value);
        match r {
            Ok(n) => {
                // dbg!(&n);
                match n {
                    // Forwarded 
                    NotificationMessage::PortValueSingle(val) => {
                        match senders.0.send(val) {
                            Ok(_) => (),
                            Err(e) =>  {
                                eprintln!("Error forwarding PortValueSingle: {:?}", e);
                            }
                        }
                    }
                    NotificationMessage::PortValueCombined(val) => {
                        match senders.1.send(val) {
                            Ok(_) => (),
                            Err(e) =>  {
                                eprintln!("Error forwarding PortValueCombined: {:?}", e);
                            }
                        }
                    }
                    NotificationMessage::HwNetworkCommands(val) => {
                        match senders.2.send(val) {
                            Ok(_) => (),
                            Err(e) =>  {
                                eprintln!("Error forwarding HwNetworkCommands: {:?}", e);
                            }
                        }
                    }
           
                    // IoDevice collection / configuration
                    NotificationMessage::HubAttachedIo(io_event) => {
                        match io_event {
                            AttachedIo{port, event} => {
                                let port_id = port;
                                match event {
                                    IoAttachEvent::AttachedIo{io_type_id, hw_rev, fw_rev} => {
                                        {
                                            let mut hub = mutex.lock().await;
                                            hub.attach_io(IoDevice::new(io_type_id, port_id))?;
                                            hub.request_port_info(port_id, InformationType::ModeInfo).await?;
                                            // hub.request_port_info(port_id, InformationType::PossibleModeCombinations).await?;
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
                                            let port = hub.connected_io().get_mut(&port_id).unwrap();
                                            port.def.set_mode_count(mode_count);
                                            port.def.set_capabilities(capabilities.0);
                                            port.def.set_modes(input_modes, output_modes);
                                            
                                            // Req combinations if LogicalCombinable or LogicalSynchronizable
                                            if ((capabilities.0 >> 2) & 1 == 1) | ((capabilities.0 >> 3) & 1 == 1) {
                                                hub.request_port_info(port_id, InformationType::PossibleModeCombinations).await?;
                                            }
                                      
                                            for mode_id in 0..mode_count {
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Name).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Raw).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Pct).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Si).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Symbol).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Mapping).await?;
                                                // hub.req_mode_info(port_id, mode_id, ModeInformationType::MotorBias).await?;          // Returns errorcode CommandNotRecognized on all devices I've tested
                                                // hub.request_mode_info(port_id, mode_id, ModeInformationType::CapabilityBits).await;  // Don't have documentation to parse this
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::ValueFormat).await?;
                                            }
                                        }
                                    }
                                    PortInformationType::PossibleModeCombinations(combs) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_valid_combos(combs);   
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
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_name(mode, name);
                                    }
                                    PortModeInformationType::RawRange{min, max } => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_raw(mode, min, max);
                                    }
                                    PortModeInformationType::PctRange{min, max } => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_pct(mode, min, max);
                                    }
                                    PortModeInformationType::SiRange{min, max } => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_si(mode, min, max);
                                    }
                                    PortModeInformationType::Symbol(symbol) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_symbol(mode, symbol);
                                    }
                                    PortModeInformationType::Mapping{input, output} => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_mapping(mode, input, output);
                                    }
                                    PortModeInformationType::MotorBias(bias) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_motor_bias(mode, bias);
                                    }
                                    // PortModeInformationType::CapabilityBits(name) => {
                                    //     let mut hub = mutex.lock().await;
                                    //     hub.connected_io().get_mut(&port_id).unwrap().set_mode_cabability(mode, name);  //set_mode_capability not implemented
                                    // }
                                    PortModeInformationType::ValueFormat(format) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io().get_mut(&port_id).unwrap().def.set_mode_valueformat(mode, format);
                                    }
                                    _ => ()
                                }
                            }

                        }
                    }
                    
                    // Not doing anything with these yet. Alerts and error messages could be useful.
                    NotificationMessage::HubProperties(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::HubActions(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::HubAlerts(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::GenericErrorMessages(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::FwLockStatus(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::PortInputFormatSingle(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::PortInputFormatCombinedmode(val) => {
                        if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    NotificationMessage::PortOutputCommandFeedback(val ) => {
                        // if DIAGNOSTICS { eprintln!("{:?}", val); }
                    }
                    _ => ()
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
        }
    }
    Ok(())  
}