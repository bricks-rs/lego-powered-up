// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Definitions for the various devices which can attach to hubs, e.g. motors

use crate::error::{Error, Result};
use crate::hubs::Port;
use crate::notifications::{HubLedMode, NotificationMessage, Power};
use async_trait::async_trait;
use btleplug::api::{Characteristic, Peripheral, WriteType};
use std::fmt::Debug;

/// Trait that any device may implement. Having a single trait covering
/// every device is probably the wrong design, and we should have better
/// abstractions for e.g. motors vs. sensors & LEDs.
#[async_trait]
pub trait Device: Debug + Send + Sync {
    type P: Peripheral;
    fn port(&self) -> Port;
    fn peripheral(&self) -> &Self::P;
    fn characteristic(&self) -> &Characteristic;
    async fn send(&mut self, msg: NotificationMessage) -> Result<()> {
        let buf = msg.serialise();
        self.peripheral()
            .write(self.characteristic(), &buf, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }
    async fn set_rgb(&mut self, _rgb: &[u8; 3]) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn start_speed(
        &mut self,
        _speed: i8,
        _max_power: Power,
    ) -> Result<()> {
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
    peripheral: P,
    characteristic: Characteristic,
    port_id: u8,
    // hub_addr: BDAddr,
    // hub_manager_tx: Sender<HubManagerMessage>,
}

#[async_trait]
impl<P: Peripheral> Device for HubLED<P> {
    type P = P;
    fn port(&self) -> Port {
        Port::HubLed
    }

    fn peripheral(&self) -> &Self::P {
        &self.peripheral
    }

    fn characteristic(&self) -> &Characteristic {
        &self.characteristic
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

    async fn set_rgb(&mut self, rgb: &[u8; 3]) -> Result<()> {
        use crate::notifications::*;

        self.rgb = *rgb;

        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: 50,
                mode: 0x01,
                delta: 0x00000001,
                notification_enabled: false,
            });
        self.send(mode_set_msg).await?;

        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColors {
                red: rgb[0],
                green: rgb[1],
                blue: rgb[2],
            },
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }
}

impl<P: Peripheral> HubLED<P> {
    pub(crate) fn new(
        peripheral: P,
        characteristic: Characteristic,
        port_id: u8,
    ) -> Self {
        let mode = HubLedMode::Rgb;
        //hub.subscribe(mode);
        Self {
            rgb: [0; 3],
            _mode: mode,
            characteristic,
            peripheral,
            port_id,
        }
    }
}

/// Struct representing a motor
#[derive(Debug, Clone)]
pub struct Motor<P: Peripheral> {
    peripheral: P,
    characteristic: Characteristic,
    port: Port,
    port_id: u8,
    // hub_addr: BDAddr,
    // hub_manager_tx: Sender<HubManagerMessage>,
}

#[async_trait]
impl<P: Peripheral> Device for Motor<P> {
    type P = P;
    fn port(&self) -> Port {
        self.port
    }

    fn peripheral(&self) -> &Self::P {
        &self.peripheral
    }

    fn characteristic(&self) -> &Characteristic {
        &self.characteristic
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

    async fn start_speed(&mut self, speed: i8, max_power: Power) -> Result<()> {
        use crate::notifications::*;

        let subcommand = PortOutputSubcommand::StartSpeed {
            speed,
            max_power,
            use_acc_profile: true,
            use_dec_profile: true,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }
}

impl<'p, P: Peripheral> Motor<P> {
    pub(crate) fn new(
        peripheral: P,
        characteristic: Characteristic,
        // hub_addr: BDAddr,
        // hub_manager_tx: Sender<HubManagerMessage>,
        port: Port,
        port_id: u8,
    ) -> Self {
        Self {
            peripheral,
            characteristic,
            port,
            port_id,
            // hub_addr,
        }
    }
}
