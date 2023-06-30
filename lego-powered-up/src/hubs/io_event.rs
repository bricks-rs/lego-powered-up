#![allow(unused)]

/// Models the hub acting on the messages parsed by
/// notification module. The 3 main messagetypes for
/// device comms are forwarded to channels for devices
/// to subscribe. Device info messagetypes are handed
/// to iodevice::definition mod. The rest are available
/// as error messages for now.
use core::pin::Pin;
use futures::stream::{Stream, StreamExt};

use btleplug::api::ValueNotification;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

use crate::error::Result;
use crate::hubs::HubNotification;
use crate::notifications::*;
use crate::IoDevice;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

// pub enum Verbosity {
//     Attached(bool),
//     Hub(bool),
//     Input(bool),
//     Output(bool),
//     Values(bool),
// }
pub struct Verbosity {
    attached: bool,
    hub: bool,
    input: bool,
    output: bool,
    values: bool,
}
impl Verbosity {
    pub fn new() -> Self {
        Self {
            attached: false,
            hub: false,
            input: false,
            output: false,
            values: false,
        }
    }
    pub fn set(&mut self, attached: bool, hub: bool, input: bool, output: bool, values: bool)  {
        self.attached = attached;
    }
}

pub async fn io_event_handler(
    mut stream: PinnedStream,
    mutex: HubMutex,
    senders: (
        Sender<PortValueSingleFormat>,
        Sender<PortValueCombinedFormat>,
        Sender<NetworkCommand>,
        Sender<HubNotification>,
        Sender<PortOutputCommandFeedbackFormat>
    ),
    // verbosity: Verbosity,
) -> Result<()> {
    // Verbosity
    const ATTACHED: bool = true;
    const HUB: bool = false;
    const INPUT: bool = false;
    const OUTPUT: bool = false;
    const _VALUES: bool = false;
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
                            Err(e) => {
                                eprintln!("No receiver? Error forwarding PortValueSingle: {:?}", e);
                            }
                        }
                    }
                    NotificationMessage::PortValueCombined(val) => {
                        match senders.1.send(val) {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("No receiver? Error forwarding PortValueCombined: {:?}", e);
                            }
                        }
                    }
                    NotificationMessage::HwNetworkCommands(val) => {
                        match senders.2.send(val) {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("No receiver? Error forwarding HwNetworkCommands: {:?}", e);
                            }
                        }
                    }
                    NotificationMessage::PortOutputCommandFeedback(val) => {
                        if OUTPUT {
                            eprintln!("{:?}", val);
                        }
                        if senders.4.receiver_count() > 0 {
                            match senders.4.send(val) {
                                Ok(_) => (),
                                Err(e) => {
                                    eprintln!("No receiver? Error forwarding PortOutputCommandFeedback: {:?}", e);
                                }
                            }
                        }
                    }

                    // IoDevice collection / configuration
                    NotificationMessage::HubAttachedIo(io_event) => {
                        let AttachedIo { port, event } = io_event;
                        {
                            let port_id = port;
                            match event {
                                IoAttachEvent::AttachedIo {
                                    io_type_id,
                                    hw_rev,
                                    fw_rev,
                                } => {
                                    {
                                        let mut hub = mutex.lock().await;
                                        hub.attach_io(IoDevice::new(
                                            io_type_id, port_id,
                                        ))?;
                                        hub.request_port_info(
                                            port_id,
                                            InformationType::ModeInfo,
                                        )
                                        // .await?;
                                        ?;
                                        // hub.request_port_info(port_id, InformationType::PossibleModeCombinations).await?; // conditional req in PortInformation-arm
                                    }
                                    if ATTACHED {
                                        eprintln!(
                                            "AttachedIo: {:?} {:?}",
                                            port_id, event
                                        );
                                    }
                                }
                                IoAttachEvent::DetachedIo {} => {
                                    {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io_mut().remove(&port_id);
                                    }
                                    if ATTACHED {
                                        eprintln!(
                                            "DetachedIo: {:?} {:?}",
                                            port_id, event
                                        );
                                    }
                                }
                                IoAttachEvent::AttachedVirtualIo {
                                    io_type_id,
                                    port_a,
                                    port_b,
                                } => {
                                    {
                                        let mut hub = mutex.lock().await;
                                        hub.attach_io(IoDevice::new(
                                            io_type_id, port_id,
                                        ))?;
                                        hub.request_port_info(
                                            port_id,
                                            InformationType::ModeInfo,
                                        )?;
                                        // .await?;
                                    }
                                    if ATTACHED {
                                        eprintln!(
                                            "AttachedVirtualIo: {:?} {:?}",
                                            port_id, event
                                        );
                                    }
                                }
                            }
                        }
                    }
                    NotificationMessage::PortInformation(val) => {
                        let PortInformationValue {
                            port_id,
                            information_type,
                        } = val;
                        {
                            let port_id = port_id;
                            match information_type {
                                    PortInformationType::ModeInfo{capabilities, mode_count, input_modes, output_modes} => {
                                        {
                                            let mut hub = mutex.lock().await;
                                            let device = hub.connected_io_mut().get_mut(&port_id).unwrap();
                                            device.def.set_mode_count(mode_count);
                                            device.def.set_capabilities(capabilities.0);
                                            device.def.set_modes(input_modes, output_modes);

                                            // Req combinations if capability LogicalCombinable
                                            if (capabilities.0 >> 2) & 1 == 1  {
                                                hub.request_port_info(port_id, InformationType::PossibleModeCombinations)?;
                                            }

                                            for mode_id in 0..mode_count {
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Name)?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Raw)?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Pct)?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Si)?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Symbol)?;
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::Mapping)?;
                                                // hub.req_mode_info(port_id, mode_id, ModeInformationType::MotorBias).await?;          // Returns errorcode CommandNotRecognized on all devices I've tested
                                                // hub.request_mode_info(port_id, mode_id, ModeInformationType::CapabilityBits).await;  // Don't have documentation to parse this
                                                hub.req_mode_info(port_id, mode_id, ModeInformationType::ValueFormat)?;
                                            }
                                        }
                                    }
                                    PortInformationType::PossibleModeCombinations(combs) => {
                                        let mut hub = mutex.lock().await;
                                        hub.connected_io_mut().get_mut(&port_id).unwrap().def.set_valid_combos(combs);                                    }
                                }
                        }
                    }
                    NotificationMessage::PortModeInformation(val) => {
                        let PortModeInformationValue {
                            port_id,
                            mode,
                            information_type,
                        } = val;
                        match information_type {
                            PortModeInformationType::Name(name) => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()
                                    .def
                                    .set_mode_name(mode, name);
                            }
                            PortModeInformationType::RawRange { min, max } => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()
                                    .def
                                    .set_mode_raw(mode, min, max);
                            }
                            PortModeInformationType::PctRange { min, max } => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()
                                    .def
                                    .set_mode_pct(mode, min, max);
                            }
                            PortModeInformationType::SiRange { min, max } => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()   // panic
                                    .def
                                    .set_mode_si(mode, min, max);
                            }
                            PortModeInformationType::Symbol(symbol) => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()   // panic
                                    .def
                                    .set_mode_symbol(mode, symbol);
                            }
                            PortModeInformationType::Mapping {
                                input,
                                output,
                            } => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()           // thread 'tokio-runtime-worker' panicked at 'called `Option::unwrap()` on a `None` value', /mnt/r/code/hus_project/api/lego-powered-up/lego-powered-up/src/hubs/io_event.rs:237:38
                                    .def
                                    .set_mode_mapping(mode, input, output);
                            }
                            PortModeInformationType::MotorBias(bias) => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()
                                    .def
                                    .set_mode_motor_bias(mode, bias);
                            }
                            // PortModeInformationType::CapabilityBits(name) => {
                            //     let mut hub = mutex.lock().await;
                            //     hub.connected_io_mut().get_mut(&port_id).unwrap().set_mode_cabability(mode, name);  //set_mode_capability not implemented
                            // }
                            PortModeInformationType::ValueFormat(format) => {
                                let mut hub = mutex.lock().await;
                                hub.connected_io_mut()
                                    .get_mut(&port_id)
                                    .unwrap()
                                    .def
                                    .set_mode_valueformat(mode, format);
                            }
                            _ => (),
                        }
                    }

                    // Forward hub notifications
                    NotificationMessage::HubProperties(val) => {
                        if HUB {
                            eprintln!("{:?}", &val);
                        }
                        match senders.3.send(HubNotification {
                            hub_property: Some(val),
                            hub_action: None,
                            hub_alert: None,
                            hub_error: None,
                        }) {
                            Ok(_) => (),
                            Err(e) => {
                                if !HUB {
                                    eprintln!("No receiver? Error forwarding HubProperties: {:?}", e)
                                };
                            }
                        }
                    }
                    NotificationMessage::HubActions(val) => {
                        // if HUB {
                            eprintln!("{:?}", &val);
                        // }
                        match senders.3.send(HubNotification {
                            hub_property: None,
                            hub_action: Some(val),
                            hub_alert: None,
                            hub_error: None,
                        }) {
                            Ok(_) => (),
                            Err(e) => {
                                if !HUB {
                                    eprintln!("No receiver? Error forwarding HubActions: {:?}", e)
                                };
                            }
                        }
                    }
                    NotificationMessage::HubAlerts(val) => {
                        if HUB {
                            eprintln!("{:?}", &val);
                        }
                        match senders.3.send(HubNotification {
                            hub_property: None,
                            hub_action: None,
                            hub_alert: Some(val),
                            hub_error: None,
                        }) {
                            Ok(_) => (),
                            Err(e) => {
                                if !HUB {
                                    eprintln!("No receiver? Error forwarding HubAlerts: {:?}", e)
                                };
                            }
                        }
                    }
                    NotificationMessage::GenericErrorMessages(val) => {
                        if HUB {
                            eprintln!("{:?}", &val);
                        }
                        match senders.3.send(HubNotification {
                            hub_property: None,
                            hub_action: None,
                            hub_alert: None,
                            hub_error: Some(val),
                        }) {
                            Ok(_) => (),
                            Err(e) => {
                                if !HUB {
                                    eprintln!("No receiver? Error forwarding GenericErrorMessages: {:?}", e)
                                };
                            }
                        }
                    }

                    // Not doing anything with these yet.
                    NotificationMessage::FwLockStatus(val) => {
                        if HUB {
                            eprintln!("{:?}", val);
                        }
                    }
                    NotificationMessage::PortInputFormatSingle(val) => {
                        if INPUT {
                            eprintln!("{:?}", val);
                        }
                    }
                    NotificationMessage::PortInputFormatCombinedmode(val) => {
                        if INPUT {
                            eprintln!("{:?}", val);
                        }
                    }
                   
                    _ => (),
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
        }
    }
    Ok(())
}
