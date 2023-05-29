#![allow(unused)]
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Definitions for the various devices which can attach to hubs, e.g. motors

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::fmt::Debug;
use std::process::ExitStatus;

use async_trait::async_trait;
use core::time::Duration;
use crate::error::{Error, Result};
use crate::hubs::Port;
use crate::notifications::{HubLedMode, NotificationMessage, Power, EndState, PortModeInformationType, InputSetupCombined};

use crate::notifications::{InputSetupSingle, PortOutputSubcommand, WriteDirectModeDataPayload, 
    PortOutputCommandFormat, StartupInfo, CompletionInfo, InformationRequest, ModeInformationRequest,
    PortInformationType, ModeInformationType, InformationType,  };

pub use crate::consts::*;

pub mod iodevice;
pub mod remote;
pub mod sensor;
pub mod motor;

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

    // Lights
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

    // Sensors
    async fn color_sensor_enable(&mut self, mode: u8, delta: u32) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn color_sensor_disable(&mut self) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }


    // Motors
    async fn start_power(&mut self, _power: Power,) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
    async fn start_power2(&mut self, _power1: Power, _power2: Power) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
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
    async fn motor_combined_sensor_enable(&mut self, mode: MotorSensorMode, speed_delta: u32, position_delta: u32) -> Result<()> {
        Err(Error::NotImplementedError(
            "Not implemented for type".to_string(),
        ))
    }
}


pub struct VisionSensor {
    // kind: IoTypeId
    peripheral: Peripheral,
    characteristic: Characteristic,
    port_id: u8,
    port: Port,
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

    pub(crate) fn newnew(
        peripheral: Peripheral,
        characteristic: Characteristic,
        port_id: u8,
    ) -> Self {
        Self {
            peripheral,
            characteristic,
            port_id,
            port: Port::Deprecated   // deprecated
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

    pub(crate) fn newnew(peripheral: Peripheral, characteristic: Characteristic, port_id: u8,) -> Self {
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

    async fn start_power(&mut self, power: Power) -> Result<()> {
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::StartPower(power) 
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
    async fn start_power2(&mut self, power1: Power, power2: Power) -> Result<()> {
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::StartPower2 {
                power1,
                power2
            } 
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
    
    async fn motor_combined_sensor_enable(&mut self, primary_mode: MotorSensorMode, speed_delta: u32, position_delta: u32) -> Result<()> {
        use crate::notifications::*;
        
        // Step 1: Lock device
        let subcommand = InputSetupCombinedSubcommand::LockLpf2DeviceForSetup {};     
        let msg =
            NotificationMessage::PortInputFormatSetupCombinedmode(InputSetupCombined {
                port_id: self.port_id,
                subcommand,
            });
        self.send(msg).await;

        // Step 2: Set up modes
        self.motor_sensor_enable(MotorSensorMode::Speed, speed_delta).await;
        // self.motor_sensor_enable(MotorSensorMode::APos, position_delta).await;      // Returns "Invalid use" of Port Input Format Setup (Single) [0x41]"
        self.motor_sensor_enable(MotorSensorMode::Pos, position_delta).await;

        // Step 3: Set up combination
        let mut sensor0_mode_nibble: u8;
        let mut sensor1_mode_nibble: u8;
        // let mut sensor2_mode_nibble: u8;
        let dataset_nibble: u8 = 0x00;     // All motor modes have 1 dataset only 
        match primary_mode {
            MotorSensorMode::Speed => {
                sensor0_mode_nibble = 0x10; // Speed 
                sensor1_mode_nibble = 0x20; // Pos
                // sensor2_mode_nibble = 0x30; // APos
            }
            MotorSensorMode::Pos => {
                sensor0_mode_nibble = 0x20; // Pos 
                sensor1_mode_nibble = 0x10; // Speed
                // sensor2_mode_nibble = 0x30; // APos
            }
            _ => {
                sensor0_mode_nibble = 0x00;  
                sensor1_mode_nibble = 0x00; 
                // sensor2_mode_nibble = 0x00; 
            }
        }
        let subcommand = InputSetupCombinedSubcommand::SetModeanddatasetCombinations { 
            combination_index: 0, 
            mode_dataset: [
                sensor0_mode_nibble + dataset_nibble, 
                sensor1_mode_nibble + dataset_nibble, 
                // sensor2_mode_nibble + dataset_nibble,
                255, 0, 0, 0, 0, 0 // 255-byte marks end, cf. comment in InputSetupCombined::serialise.
            ] 
        };     
        let msg =
            NotificationMessage::PortInputFormatSetupCombinedmode(InputSetupCombined {
                port_id: self.port_id,
                subcommand,
            });
        self.send(msg).await;
 
        // Step 4: Unlock device and enable multi updates 
        let subcommand = InputSetupCombinedSubcommand::UnlockAndStartMultiEnabled {};     
        let msg =
            NotificationMessage::PortInputFormatSetupCombinedmode(InputSetupCombined {
                port_id: self.port_id,
                subcommand,
            });
        self.send(msg).await;

        Ok(())
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

    pub(crate) fn newnew(
        peripheral: Peripheral,
        characteristic: Characteristic,
        port_id: u8,
    ) -> Self {
        Self {
            peripheral,
            characteristic,
            port_id,
            port: Port::Deprecated ,  // deprecated
            status: MotorStatus::new(),
        }
    }
}


