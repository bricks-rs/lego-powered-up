#![allow(unused)]

use core::pin::Pin;
use futures::stream::{Stream, StreamExt};

use std::fmt::Debug;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use btleplug::api::ValueNotification;

use crate::IoDevice;
use crate::error::{Error, Result, OptionContext};
use crate::notifications::*;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;


  // // Musings on notfication-message parsing and handling
        // I realized that the design where there is a handler like this for each enabled port (where an iodevice has been
        // req'd / cloned from hub and a handler started) means sending each message thru NotificationMessage::parse once 
        // for every task. Better to split out by messagetype => broadcast-channel. (Setting up handler with dedicated parsing is fine is the need 
        // is felt for an application.) 
        //
        // So: Somewhere (in hub?) a hashmap (in struct for methods) that maps Notification-message type to a a sender/receiver pair).
        // Then a main handler (for each hub-stream) get a clone of that? Or just the senders? 
        // I'll start with this function as main handler (let it also handle hub-related messages for now.) 
        //
        // Those messagetypes that have a port in their value could be further split to avoid multiple processing of those.
        // Perhaps not the config/information ones but PortValueSingle, PortValueCombined and PortOutputCommandFeedback have
        // the potential to be numerous. 
        // Although this needs config information. Lets 
        // Alt. 1: Create subtasks to receive each of the 3 to a channel by port. Problem: Would need to spawn/create sender when port enabled (or configured)
        // Vs. the messagetypes being known at compile time.
        // Alt. 2: Main handler has senders for each of the 3 by port. Again this would need to be dynamic. 
        // We'll start by splitting out those three to to senders i think.  


//Deprecated
#[derive(Debug, Default, Clone)]
pub struct ChannelNotification {
    pub portvaluesingle: Option<PortValueSingleFormat>,
    pub portvaluescombined: Option<PortValueCombinedFormat>,
    pub portoutputcommandfeedback: Option<PortOutputCommandFeedbackFormat>
}

// This was an idea for generic tasking, went instead with separate functions
// for the types (trait GenericSensor in devices::sensor). That could surely
// be bettered with generic types, but low priority right now. 
// #[derive(Debug, Default, Clone)]
// pub struct ValWrap {
//     pub uint8: Option<Vec<u8>>,
//     pub uint16: Option<Vec<u16>>,
//     pub uint32: Option<Vec<u32>>,
//     pub float32: Option<Vec<f32>>,
// }
// impl ValWrap {
//     pub fn new() -> Self {
//         Self {
//             uint8: None,
//             uint16: None,
//             uint32: None,
//             float32: None, 
//         }
//     }
// }

pub async fn io_event_handler(mut stream: PinnedStream, mutex: HubMutex, _hub_name: String, 
                            tx_singlevalue: broadcast::Sender<PortValueSingleFormat>,
                            tx_combinedvalue: broadcast::Sender<PortValueCombinedFormat>,
                            tx_networkcmd: broadcast::Sender<NetworkCommand>
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
                        tx_singlevalue.send(val).expect("Error forwarding PortValueSingle");
                    }
                    NotificationMessage::PortValueCombined(val) => {
                        tx_combinedvalue.send(val).expect("Error forwarding PortValueCombined");
                    }
                    NotificationMessage::HwNetworkCommands(val) => {
                        tx_networkcmd.send(val).expect("Error forwarding NetworkCommands");
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

                                            // let p = hub.peripheral().clone();
                                            // let c = hub.characteristic().clone();
                                            // hub.attach_io(
                                            //     IoDevice::new_with_handles(
                                            //         io_type_id, port_id, p, c));
                                            
                                            hub.request_port_info(port_id, InformationType::ModeInfo).await?;
                                            hub.request_port_info(port_id, InformationType::PossibleModeCombinations).await?;
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
                                            port.set_mode_count(mode_count);
                                            port.set_capabilities(capabilities.0);
                                            port.set_modes(input_modes, output_modes);
                                      
                                            // let count = 
                                            for mode_id in 0..mode_count {
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Name).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Raw).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Pct).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Si).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Symbol).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Mapping).await?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::MotorBias).await?;
                                                // hub.request_mode_info(port_id, mode_id, ModeInformationType::CapabilityBits).await;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::ValueFormat).await?;
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