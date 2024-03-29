//! Models the hub acting on the messages parsed by
//! notification module. The 3 main messagetypes for
//! device comms are forwarded to channels for devices
//! to subscribe. Device info messagetypes are handed
//! to iodevice::definition mod. The rest are available
//! as error messages for now.

use core::pin::Pin;
use futures::stream::{Stream, StreamExt};

use btleplug::api::ValueNotification;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::error::Result;
use crate::hubs::HubNotification;
use crate::notifications::*;

use super::Channels;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

pub struct Verbosity {
    attached: bool,
    _hub: bool,
    _input: bool,
    _output: bool,
    _values: bool,
}
impl Verbosity {
    pub fn new() -> Self {
        Self {
            attached: true,
            _hub: false,
            _input: false,
            _output: true,
            _values: false,
        }
    }
    pub fn set(
        &mut self,
        attached: bool,
        _hub: bool,
        _input: bool,
        _output: bool,
        _values: bool,
    ) {
        self.attached = attached;
    }
}
impl Default for Verbosity {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn io_event_handler(
    mut stream: PinnedStream,
    mutex: HubMutex,
    senders: Channels,
    cancel: CancellationToken,
) -> Result<()> {
    if senders.networkcmd_sender.is_none()
        | senders.combinedvalue_sender.is_none()
        | senders.hubnotification_sender.is_none()
        | senders.singlevalue_sender.is_none()
        | senders.commandfeedback_sender.is_none()
    {
        return Err(crate::Error::HubError("Sender was none".into()));
    }
    let combinedvalue_sender = senders.combinedvalue_sender.unwrap();
    let commandfeedback_sender = senders.commandfeedback_sender.unwrap();
    let hubnotification_sender = senders.hubnotification_sender.unwrap();
    let networkcmd_sender = senders.networkcmd_sender.unwrap();
    let singlevalue_sender = senders.singlevalue_sender.unwrap();

    // Verbosity
    const ATTACHED: bool = true;
    const HUB: bool = false;
    const INPUT: bool = false;
    const OUTPUT: bool = true;
    const _VALUES: bool = false;
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            }

            Some(data) = stream.next() => {
            let n = match NotificationMessage::parse(&data.value) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    continue;
                }
            };

            match n {
                // Forwarded
                NotificationMessage::PortValueSingle(val) => {
                    match singlevalue_sender.send(val) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("No receiver? Error forwarding PortValueSingle: {:?}", e);
                        }
                    }
                }
                NotificationMessage::PortValueCombined(val) => {
                    match combinedvalue_sender.send(val) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("No receiver? Error forwarding PortValueCombined: {:?}", e);
                        }
                    }
                }
                NotificationMessage::HwNetworkCommands(val) => {
                    if networkcmd_sender.receiver_count() > 0 {
                        match networkcmd_sender.send(val) {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("No receiver? Error forwarding HwNetworkCommands: {:?}", e);
                            }
                        }
                    }
                }
                NotificationMessage::PortOutputCommandFeedback(val) => {
                    if OUTPUT {
                        eprintln!("{:?}", val);
                    }
                    if commandfeedback_sender.receiver_count() > 0 {
                        match commandfeedback_sender.send(val) {
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
                                hw_rev:_,
                                fw_rev:_,
                            } => {
                                {
                                    let mut hub = mutex.lock().await;
                                    hub.attach_io(io_type_id, port_id)?;
                                    hub.request_port_info(
                                        port_id,
                                        InformationType::ModeInfo,
                                    )
                                    .await?;
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
                                port_a:_,
                                port_b:_,
                            } => {
                                {
                                    let mut hub = mutex.lock().await;
                                    hub.attach_io(io_type_id, port_id)?;
                                    hub.request_port_info(
                                        port_id,
                                        InformationType::ModeInfo,
                                    ).await?;
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
                                            hub.request_port_info(port_id, InformationType::PossibleModeCombinations).await?;
                                        }

                                        for mode_id in 0..mode_count {
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::Name).await?;
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::Raw).await?;
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::Pct).await?;
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::Si).await?;
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::Symbol).await?;
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::Mapping).await?;
                                            hub.req_mode_info(port_id, mode_id, ModeInformationType::ValueFormat).await?;
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
                                .unwrap()
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
                    match hubnotification_sender.send(HubNotification {
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
                    if HUB {
                        eprintln!("{:?}", &val);
                    }
                    match hubnotification_sender.send(HubNotification {
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
                    match hubnotification_sender.send(HubNotification {
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
                    match hubnotification_sender.send(HubNotification {
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


            }}
        }
    }
    Ok(())
}
