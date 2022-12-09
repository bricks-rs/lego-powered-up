// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Definitions for the various devices which can attach to hubs, e.g. motors

use crate::error::{Error, Result};
use crate::hubs::Port;
use crate::notifications::{HubLedMode, Power};
use btleplug::api::{BDAddr, Peripheral};
use std::fmt::Debug;
use std::sync::Arc;

/// Trait that any device may implement. Having a single trait covering
/// every device is probably the wrong design, and we should have better
/// abstractions for e.g. motors vs. sensors & LEDs.
pub trait Device<'p>: Debug + Send + Sync {
    fn port(&self) -> Port;
    // fn send(&mut self, _msg: NotificationMessage) -> Result<()>;
    fn set_rgb(&mut self, _rgb: &[u8; 3]) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    fn start_speed(&mut self, _speed: i8, _max_power: Power) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
}

// / Create a device manager for the given port
// pub(crate) fn create_device<'p, P: Peripheral + 'p>(
//     peripheral: Arc<P>,
//     port_id: u8,
//     port_type: Port,
//     hub_addr: BDAddr,
//     // hub_manager_tx: Sender<HubManagerMessage>,
// ) -> Box<dyn Device<'p> + Send + Sync + 'p> {
//     match port_type {
//         Port::HubLed => {
//             let dev = HubLED::new(peripheral, port_id);
//             Box::new(dev)
//         }
//         Port::A | Port::B | Port::C | Port::D => {
//             let dev = Motor::new(peripheral, hub_addr, port_type, port_id);
//             Box::new(dev)
//         }
//         _ => todo!(),
//     }
// }

/// Struct representing a Hub LED
#[derive(Debug, Clone)]
pub struct HubLED<P: Peripheral> {
    /// RGB colour value
    rgb: [u8; 3],
    _mode: HubLedMode,
    peripheral: Arc<P>,
    port_id: u8,
    // hub_addr: BDAddr,
    // hub_manager_tx: Sender<HubManagerMessage>,
}

impl<P: Peripheral> Device<'_> for HubLED<P> {
    fn port(&self) -> Port {
        Port::HubLed
    }

    // fn send(&mut self, msg: NotificationMessage) -> Result<()> {
    //     let (tx, rx) = bounded::<Result<()>>(1);
    //     self.hub_manager_tx.send(HubManagerMessage::SendToHub(
    //         self.hub_addr,
    //         msg,
    //         tx,
    //     ))?;
    //     rx.recv()?
    // }

    fn set_rgb(&mut self, rgb: &[u8; 3]) -> Result<()> {
        // use crate::notifications::*;

        self.rgb = *rgb;

        // let mode_set_msg =
        //     NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
        //         port_id: 50,
        //         mode: 0x01,
        //         delta: 0x00000001,
        //         notification_enabled: false,
        //     });
        // self.send(mode_set_msg)?;

        // let subcommand = PortOutputSubcommand::WriteDirectModeData(
        //     WriteDirectModeDataPayload::SetRgbColors {
        //         red: rgb[0],
        //         green: rgb[1],
        //         blue: rgb[2],
        //     },
        // );

        // let msg =
        //     NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
        //         port_id: self.port_id,
        //         startup_info: StartupInfo::ExecuteImmediately,
        //         completion_info: CompletionInfo::NoAction,
        //         subcommand,
        //     });
        // self.send(msg)
        Ok(())
    }
}

impl<P: Peripheral> HubLED<P> {
    pub(crate) fn new(
        peripheral: Arc<P>,
        // hub_addr: BDAddr,
        // hub_manager_tx: Sender<HubManagerMessage>,
        port_id: u8,
    ) -> Self {
        let mode = HubLedMode::Rgb;
        //hub.subscribe(mode);
        Self {
            rgb: [0; 3],
            _mode: mode,
            peripheral,
            port_id,
            // hub_addr,
            // hub_manager_tx,
        }
    }
}

/// Struct representing a motor
#[derive(Debug, Clone)]
pub struct Motor<P: Peripheral> {
    peripheral: Arc<P>,
    port: Port,
    port_id: u8,
    // hub_addr: BDAddr,
    // hub_manager_tx: Sender<HubManagerMessage>,
}

impl<P: Peripheral> Device<'_> for Motor<P> {
    fn port(&self) -> Port {
        self.port
    }

    // fn send(&mut self, msg: NotificationMessage) -> Result<()> {
    //     let (tx, rx) = bounded::<Result<()>>(1);
    //     self.hub_manager_tx.send(HubManagerMessage::SendToHub(
    //         self.hub_addr,
    //         msg,
    //         tx,
    //     ))?;
    //     rx.recv()?
    // }

    fn start_speed(&mut self, speed: i8, max_power: Power) -> Result<()> {
        // use crate::notifications::*;

        // let subcommand = PortOutputSubcommand::StartSpeed {
        //     speed,
        //     max_power,
        //     use_acc_profile: true,
        //     use_dec_profile: true,
        // };
        // let msg =
        //     NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
        //         port_id: self.port_id,
        //         startup_info: StartupInfo::ExecuteImmediately,
        //         completion_info: CompletionInfo::NoAction,
        //         subcommand,
        //     });
        // self.send(msg)
        Ok(())
    }
}

impl<P: Peripheral> Motor<P> {
    pub(crate) fn new(
        peripheral: Arc<P>,
        // hub_addr: BDAddr,
        // hub_manager_tx: Sender<HubManagerMessage>,
        port: Port,
        port_id: u8,
    ) -> Self {
        Self {
            peripheral,
            port,
            port_id,
            // hub_addr,
        }
    }
}
