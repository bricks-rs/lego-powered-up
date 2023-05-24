#![allow(unused)]
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Definitions for the various devices which can attach to hubs, e.g. motors

use crate::error::{Error, Result};
use crate::hubs::Port;
use crate::notifications::{HubLedMode, NotificationMessage, Power, EndState, PortModeInformationType};
use async_trait::async_trait;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::fmt::Debug;
use std::process::ExitStatus;

use crate::notifications::{InputSetupSingle, PortOutputSubcommand, WriteDirectModeDataPayload, 
    PortOutputCommandFormat, StartupInfo, CompletionInfo, InformationRequest, ModeInformationRequest,
    PortInformationType, ModeInformationType, InformationType, };
pub enum MotorSensorMode {
    Speed = 0x1,
    Angle = 0x2,
}


/// Trait that any d
/// evice may implement. Having a single trait covering
/// every device is probably the wrong design, and we should have better
/// abstractions for e.g. motors vs. sensors & LEDs.
#[async_trait]
pub trait Device: Debug + Send + Sync {
    fn port(&self) -> Port;
    fn peripheral(&self) -> &Peripheral;
    fn characteristic(&self) -> &Characteristic;

    async fn send(&mut self, msg: NotificationMessage) -> Result<()> {
        let buf = msg.serialise();
        self.peripheral()
            .write(self.characteristic(), &buf, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }

    // Port information
    async fn request_port_info(&mut self, infotype: InformationType) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn request_mode_info(&mut self, mode: u8, infotype: ModeInformationType) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }

    // Hub internal devices
    async fn set_rgb(&mut self, _rgb: &[u8; 3]) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }

    // Remote
    async fn remote_buttons_enable(&mut self, mode: u8, delta: u32) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn remote_buttons_disable(&mut self) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }


    // Motors
    async fn start_speed(&mut self, _speed: i8, _max_power: Power,) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn start_speed_for_degrees(&mut self, _degrees: i32, _speed: i8, _max_power: Power, _end_state: EndState ) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn goto_absolute_position(&mut self, _abspos: i32, _speed: i8, _max_power: Power, _end_state: EndState) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn preset_encoder(&mut self, _position: i32, ) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn set_acc_time(&mut self, _time: i16, _profile_number: i8) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn set_dec_time(&mut self, _time: i16, _profile_number: i8) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn motor_sensor_enable(&mut self, mode: MotorSensorMode, delta: u32) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }

    async fn motor_sensor_disable(&mut self) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }




}


/// Struct representing a remote button cluster
#[derive(Debug, Clone)]
pub struct RemoteButtons {
    peripheral: Peripheral,
    characteristic: Characteristic,
    port_id: u8,
    port: Port,
}

#[async_trait]
impl Device for RemoteButtons {
    fn port(&self) -> Port {
        self.port
    }
    

    fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }
    fn characteristic(&self) -> &Characteristic {
        &self.characteristic
    }
    async fn request_port_info(&mut self, infotype: InformationType) -> Result<()> {
        let msg =
        NotificationMessage::PortInformationRequest(InformationRequest {
            port_id: self.port_id,
            information_type: infotype,
        });
    self.send(msg).await
    }
    async fn request_mode_info(&mut self, mode: u8, infotype: ModeInformationType) -> Result<()> {
        let msg =
        NotificationMessage::PortModeInformationRequest(ModeInformationRequest {
            port_id: self.port_id,
            mode,
            information_type: infotype,
        });
    self.send(msg).await
    }
    async fn remote_buttons_enable(&mut self, mode: u8, delta: u32) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port_id,
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        self.send(mode_set_msg).await
    }
    async fn remote_buttons_disable(&mut self) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port_id,
                mode: 0,
                delta: u32::MAX,
                notification_enabled: false,
            });
        self.send(mode_set_msg).await
    }
}
impl RemoteButtons {
    pub(crate) fn new(
        peripheral: Peripheral,
        characteristic: Characteristic,
        port: Port,
        port_id: u8,
    ) -> Self {
        Self {
            peripheral,
            characteristic,
            port,
            port_id,
        }
    }
}



/// Struct representing a Hub LED
#[derive(Debug, Clone)]
pub struct HubLED {
    /// RGB colour value
    rgb: [u8; 3],
    _mode: HubLedMode,
    peripheral: Peripheral,
    characteristic: Characteristic,
    port_id: u8,
}

#[async_trait]
impl Device for HubLED {
    fn port(&self) -> Port {
        Port::HubLed
    }



    fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    fn characteristic(&self) -> &Characteristic {
        &self.characteristic
    }

    async fn set_rgb(&mut self, rgb: &[u8; 3]) -> Result<()> {
        // use crate::notifications::*;

        self.rgb = *rgb;

        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port_id,
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

impl HubLED {
    pub(crate) fn new(
        peripheral: Peripheral,
        characteristic: Characteristic,
        port_id: u8,
    ) -> Self {
        let mode = HubLedMode::Rgb;
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
pub struct Motor {
    peripheral: Peripheral,
    characteristic: Characteristic,
    port: Port,
    port_id: u8,
    status: MotorStatus,
}
#[derive(Debug, Clone)]
pub struct MotorStatus {
    speed: i8,
    position: i16,
}
impl MotorStatus {
    fn new() -> Self {
        Self {
            speed: 0,
            position: 0,
        }
    }
}

#[async_trait]
impl Device for Motor {
    fn port(&self) -> Port {
        self.port
    }

    fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    fn characteristic(&self) -> &Characteristic {
        &self.characteristic
    }

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

    async fn start_speed_for_degrees(&mut self, degrees: i32, speed: i8, max_power: Power, end_state: EndState ) -> Result<()> {
        use crate::notifications::*;

        let subcommand = PortOutputSubcommand::StartSpeedForDegrees {
            degrees,
            speed,
            max_power,
            end_state,
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

    async fn goto_absolute_position(&mut self, abspos: i32, speed: i8, max_power: Power, end_state: EndState ) -> Result<()> {
        use crate::notifications::*;

        let subcommand = PortOutputSubcommand::GotoAbsolutePosition { 
            abs_pos: abspos,
            speed,
            max_power,
            end_state,
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
    
    async fn preset_encoder(&mut self, position: i32) -> Result<()> {
        use crate::notifications::*;

        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: 0x01, // Port B
                mode: 0x01,
                delta: 0x00000001,
                notification_enabled: false,
            });
        self.send(mode_set_msg).await?;

        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::PresetEncoder(position),);

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }

    async fn set_acc_time(&mut self, time: i16, profile_number: i8) -> Result<()> {
        use crate::notifications::*;

        let subcommand = PortOutputSubcommand::SetAccTime { time, profile_number, };

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }

    async fn set_dec_time(&mut self, time: i16, profile_number: i8) -> Result<()> {
        use crate::notifications::*;

        let subcommand = PortOutputSubcommand::SetDecTime { time, profile_number, };

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }

    async fn motor_sensor_enable(&mut self, mode: MotorSensorMode, delta: u32) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port_id,
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        self.send(mode_set_msg).await
    }

    async fn motor_sensor_disable(&mut self) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port_id,
                mode: 0,
                delta: u32::MAX,
                notification_enabled: false,
            });
        self.send(mode_set_msg).await
    }
    

    
}

impl Motor {
    pub(crate) fn new(
        peripheral: Peripheral,
        characteristic: Characteristic,
        port: Port,
        port_id: u8,
    ) -> Self {
        Self {
            peripheral,
            characteristic,
            port,
            port_id,
            status: MotorStatus::new(),
        }
    }
}


