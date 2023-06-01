use async_trait::async_trait;
use core::fmt::Debug;

use btleplug::api::{Characteristic};
use btleplug::platform::Peripheral;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::error::{Error, OptionContext, Result};
use crate::notifications::NotificationMessage;
use crate::notifications::InputSetupSingle;
use crate::notifications::{ WriteDirectModeDataPayload, WriteDirectPayload};
use crate::notifications::{PortOutputSubcommand, PortOutputCommandFormat, StartupInfo, CompletionInfo};
use crate::consts::{MotorSensorMode};
use crate::notifications::{InputSetupCombined, PortInputFormatCombinedFormat, InputSetupCombinedSubcommand};
use crate::notifications::{PortValueSingleFormat, PortValueCombinedFormat};
pub use crate::notifications::{Power, EndState};


// #[derive(Debug, Copy, Clone)]
// pub enum MotorState{
//     Speed(i8),
//     Pos(i32),
//     Apos(i32)
// }

#[async_trait]
pub trait EncoderMotor: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;
    fn check(&self) -> Result<()>;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn get_rx_combined(&self) -> Result<broadcast::Receiver<PortValueCombinedFormat>>;

    // Settings
    async fn preset_encoder(&self, position: i32) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: 0x01, // Port B
                mode: 0x01,
                delta: 0x00000001,
                notification_enabled: false,
            });
            crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await?;

        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::PresetEncoder(position),);

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }

    async fn set_acc_time(&self, time: i16, profile_number: i8) -> Result<()> {
        let subcommand = PortOutputSubcommand::SetAccTime { time, profile_number, };

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }

    async fn set_dec_time(&self, time: i16, profile_number: i8) -> Result<()> {
        let subcommand = PortOutputSubcommand::SetDecTime { time, profile_number, };

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }

    // Commands
    async fn start_power(&self, power: Power) -> Result<()> {
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::StartPower(power) 
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }
    async fn start_power2(&self, power1: Power, power2: Power) -> Result<()> {
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::StartPower2 {
                power1,
                power2
            } 
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }
    async fn start_speed(&self, speed: i8, max_power: Power) -> Result<()> {
        let subcommand = PortOutputSubcommand::StartSpeed {
            speed,
            max_power,
            use_acc_profile: true,
            use_dec_profile: true,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
            crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }
    async fn start_speed_for_degrees(&self, degrees: i32, speed: i8, max_power: Power, end_state: EndState ) -> Result<()> {
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
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
            crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await
    }

    // Encoder sensor data 
    async fn motor_sensor_enable(&self, mode: MotorSensorMode, delta: u32) -> Result<()> {
        self.check();
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await
    }
    async fn motor_sensor_disable(&self) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: 0,
                delta: u32::MAX,
                notification_enabled: false,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await
    }
    async fn motor_combined_sensor_enable(&self, primary_mode: MotorSensorMode, speed_delta: u32, position_delta: u32) 
                                            -> Result<(broadcast::Receiver<Vec<u8>>, JoinHandle<()> )> {
        // Step 1: Lock device
        let subcommand = InputSetupCombinedSubcommand::LockLpf2DeviceForSetup {};     
        let msg =
            NotificationMessage::PortInputFormatSetupCombinedmode(InputSetupCombined {
                port_id: self.port(),
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await;

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
                port_id: self.port(),
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await;
 
        // Step 4: Unlock device and enable multi updates 
        let subcommand = InputSetupCombinedSubcommand::UnlockAndStartMultiEnabled {};     
        let msg =
            NotificationMessage::PortInputFormatSetupCombinedmode(InputSetupCombined {
                port_id: self.port(),
                subcommand,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), msg).await;


        // Set up channel
        let port_id = self.port();
        let (tx, mut rx) = broadcast::channel::<Vec<u8>>(8);
        match self.get_rx_combined() {
            Ok(mut rx_from_main) => { 
                let task = tokio::spawn(async move {
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        tx.send(data.data);
                        
                        // let converted_data = data.data.into_iter().map(|x| x as i8).collect();
                        // tx.send(converted_data);
                    }
                });

                Ok((rx, task))
            }
            _ => { Err(Error::NoneError((String::from("Something went wrong")))) }
        }


    }    

}



