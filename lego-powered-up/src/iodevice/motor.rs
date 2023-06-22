/// Support for the Powered Up encoder motors, aka. tacho motors.
/// The ones I've had available for testing are:
/// https://rebrickable.com/parts/22169/motor-large-powered-up/
/// https://rebrickable.com/parts/22172/motor-xl-powered-up/
/// And the internal motors in: https://rebrickable.com/parts/26910/hub-move-powered-up-6-x-16-x-4/
/// The start_power commands should work with train motors.
use async_trait::async_trait;
use btleplug::{api::Characteristic, platform::Peripheral};
use core::fmt::Debug;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::error::{Error, Result};
use crate::notifications::InputSetupSingle;
use crate::notifications::NotificationMessage;
use crate::notifications::WriteDirectModeDataPayload;
use crate::notifications::{
    CompletionInfo, PortOutputCommandFormat, PortOutputSubcommand, StartupInfo,
};
use crate::notifications::{InputSetupCombined, InputSetupCombinedSubcommand};
use crate::notifications::{PortValueCombinedFormat, PortValueSingleFormat};

pub use crate::consts::MotorSensorMode;
pub use crate::notifications::{EndState, Power};
use std::sync::Arc;


// #[derive(Debug, Copy, Clone)]
// pub enum MotorState{
//     Speed(i8),
//     Pos(i32),
//     Apos(i32)
// }

#[async_trait]
pub trait EncoderMotor: Debug + Send + Sync {
    // Motor settings
    async fn preset_encoder(&self, position: i32) -> Result<()> {
        // let msg =
        //     NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
        //         port_id: 0x01, // Port B
        //         mode: 0x01,
        //         delta: 0x00000001,
        //         notification_enabled: false,
        //     });
        // let tokens = self.tokens();
        // crate::hubs::send(tokens.0, tokens.1, msg).await.expect("Error while setting mode");

        self.check()?;
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::PresetEncoder(position),
        );
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg).await
    }

    async fn set_acc_time(&self, time: i16, profile_number: i8) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::SetAccTime {
            time,
            profile_number,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg).await
    }

    async fn set_dec_time(&self, time: i16, profile_number: i8) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::SetDecTime {
            time,
            profile_number,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg).await
    }

    // Commands
    // To do: "2" variants of all commands, except done: start_power2
    async fn start_power(&self, power: Power) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::StartPower(power),
        );
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg).await
    }
    async fn start_power2(&self, power1: Power, power2: Power) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::StartPower2 { power1, power2 };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg).await
    }
    async fn start_speed(&self, speed: i8, max_power: u8) -> Result<()> {
        self.check()?;
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
        self.commit(msg).await
    }
    async fn start_speed_for_degrees(
        &self,
        degrees: i32,
        speed: i8,
        max_power: u8,
        end_state: EndState,
    ) -> Result<()> {
        self.check()?;
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
        self.commit(msg).await
    }
    fn start_speed_for_degrees2(
        &self,
        degrees: i32,
        speed: i8,
        max_power: u8,
        end_state: EndState,
    ) -> Result<()> {
        self.check()?;
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
        self.commit2(msg)
    }
    async fn start_speed_for_time(
        &self,
        time: i16,
        speed: i8,
        max_power: u8,
        end_state: EndState,
    ) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::StartSpeedForTime {
            time,
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
        self.commit(msg).await
    }
    async fn goto_absolute_position(
        &self,
        abs_pos: i32,
        speed: i8,
        max_power: u8,
        end_state: EndState,
    ) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::GotoAbsolutePosition {
            abs_pos,
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
        self.commit(msg).await
    }

    // Encoder sensor data
    async fn motor_sensor_enable(
        &self,
        mode: MotorSensorMode,
        delta: u32,
    ) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        self.commit(msg).await
    }
    async fn motor_sensor_disable(&self) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: 0,
                delta: u32::MAX,
                notification_enabled: false,
            });
        self.commit(msg).await
    }
    async fn motor_combined_sensor_enable(
        &self,
        primary_mode: MotorSensorMode,
        speed_delta: u32,
        position_delta: u32,
    ) -> Result<(broadcast::Receiver<Vec<u8>>, JoinHandle<()>)> {
        self.check()?;
        // Step 1: Lock device
        let subcommand =
            InputSetupCombinedSubcommand::LockLpf2DeviceForSetup {};
        let msg = NotificationMessage::PortInputFormatSetupCombinedmode(
            InputSetupCombined {
                port_id: self.port(),
                subcommand,
            },
        );
        self.commit(msg).await?;

        // Step 2: Set up modes
        self.motor_sensor_enable(MotorSensorMode::Speed, speed_delta)
            .await?;
        // self.motor_sensor_enable(MotorSensorMode::APos, position_delta).await;    // Availablie on TechnicLinear motors, not on InternalTacho (MoveHub)
        self.motor_sensor_enable(MotorSensorMode::Pos, position_delta)
            .await?; // POS available on either

        // Step 3: Set up combination
        let sensor0_mode_nibble: u8;
        let sensor1_mode_nibble: u8;
        // let mut sensor2_mode_nibble: u8;
        let dataset_nibble: u8 = 0x00; // All motor modes have 1 dataset only
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
        let subcommand =
            InputSetupCombinedSubcommand::SetModeanddatasetCombinations {
                combination_index: 0,
                mode_dataset: [
                    sensor0_mode_nibble + dataset_nibble,
                    sensor1_mode_nibble + dataset_nibble,
                    // sensor2_mode_nibble + dataset_nibble,
                    255,
                    0,
                    0,
                    0,
                    0,
                    0, // 255-byte marks end, cf. comment in InputSetupCombined::serialise.
                ],
            };
        let msg = NotificationMessage::PortInputFormatSetupCombinedmode(
            InputSetupCombined {
                port_id: self.port(),
                subcommand,
            },
        );
        self.commit(msg).await?;

        // Step 4: Unlock device and enable multi updates
        let subcommand =
            InputSetupCombinedSubcommand::UnlockAndStartMultiEnabled {};
        let msg = NotificationMessage::PortInputFormatSetupCombinedmode(
            InputSetupCombined {
                port_id: self.port(),
                subcommand,
            },
        );
        self.commit(msg).await?;

        // Set up channel
        let port_id = self.port();
        let (tx, rx) = broadcast::channel::<Vec<u8>>(8);
        match self.get_rx_combined() {
            Ok(mut rx_from_main) => {
                let task = tokio::spawn(async move {
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        tx.send(data.data).expect("Error sending");

                        // let converted_data = data.data.into_iter().map(|x| x as i8).collect();
                        // tx.send(converted_data);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError(String::from("Something went wrong"))),
        }
    }

    /// Device trait boilerplate
    fn port(&self) -> u8;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn get_rx_combined(
        &self,
    ) -> Result<broadcast::Receiver<PortValueCombinedFormat>>;
    fn tokens(&self) -> (&Peripheral, &Characteristic);
    fn check(&self) -> Result<()>;
    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
        match crate::hubs::send(self.tokens(), msg).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn commit2(&self, msg: NotificationMessage) -> Result<()> {
        match crate::hubs::send2(self.tokens2(), msg) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn tokens2(&self) -> (Arc<Peripheral>, Arc<Characteristic>);
}
