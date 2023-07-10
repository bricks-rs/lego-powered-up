/// Support for the Powered Up encoder motors, aka. tacho motors.
/// The ones I've had available for testing are:
/// https://rebrickable.com/parts/22169/motor-large-powered-up/
/// https://rebrickable.com/parts/22172/motor-xl-powered-up/
/// And the internal motors in: https://rebrickable.com/parts/26910/hub-move-powered-up-6-x-16-x-4/
/// The start_power commands should work with train motors.
use async_trait::async_trait;
use core::fmt::Debug;
// use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::device_trait;
use crate::hubs::Tokens;
use crate::error::{Error, Result};
use crate::notifications::{InputSetupSingle, PortOutputCommandFeedbackFormat, FeedbackMessage};
use crate::notifications::NotificationMessage;
use crate::notifications::WriteDirectModeDataPayload;
use crate::notifications::{
    CompletionInfo, PortOutputCommandFormat, PortOutputSubcommand, StartupInfo,
};
use crate::notifications::{InputSetupCombined, InputSetupCombinedSubcommand};
use crate::notifications::{PortValueCombinedFormat, PortValueSingleFormat};

pub use crate::consts::MotorSensorMode;
pub use crate::notifications::{EndState, Power};

/// State model of a command receiver. 
/// https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#buffering-state-machine
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct CmdReceiverState {
    pub state: BufferState,

    // The progress is implied by bufferstate, so this isn't really needed? 
    // pub progress: CmdProgress,

    // 1 or 2 commands discarded. This happens when a command is sent with StartupInfo::ExecuteImmediately
    // (the other alt. is BufferIfNecessary) when state was BusyEmpty (discards cmd in progress) or
    // BusyFull (discards cmd in progress and queued command.) The queue can hold 1 command only.
    pub discarded: bool,           
}
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub enum BufferState {
    #[default] Idle,               // Nothing in progress, buffer empty. (“Idle”)
    BusyEmpty,                     // Command in progress, buffer empty (“Busy/Empty”)
    BusyFull                       // Command in progress, buffer full (“Busy/Full”)
}
// #[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
// pub enum CmdProgress {
//     InProgress,
//     #[default] Completed,
// }

device_trait!(EncoderMotor, [
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;,
    fn get_rx_combined(&self) -> Result<broadcast::Receiver<PortValueCombinedFormat>>;,
    fn get_rx_feedback(&self) -> Result<broadcast::Receiver<PortOutputCommandFeedbackFormat>>;,

    /// Set up handling of command feedback notifications
    // This supports only single motors for now, synced motors is TODO 
    fn cmd_feedback_handler(
        &self,
    ) -> Result<(broadcast::Receiver<CmdReceiverState>, JoinHandle<()>)> {
        let port_id = self.port();
        // Set up channel
        let (tx, rx) = broadcast::channel::<CmdReceiverState>(16);
        let mut rx_from_main = self
            .get_rx_feedback()
            .expect("CmdReceiverState sender not in device cache");
        let task = tokio::spawn(async move {
            while let Ok(data) = rx_from_main.recv().await {
                match data {
                    PortOutputCommandFeedbackFormat {msg1, .. } if msg1.port_id == port_id => {
                        // println!("LPU_cmdfb: {:?}", &data);
                        #[allow(clippy::match_single_binding)]
                        match msg1  {
                            FeedbackMessage { port_id:_, empty_cmd_in_progress, empty_cmd_completed:_ , discarded, idle:_, busy_full } => {
                                // Is it correct that the fields 'empty_cmd_completed' and 'idle' are redundant?
                                
                                // let mut progress = CmdProgress::Completed;
                                // if empty_cmd_in_progress | busy_full { progress = CmdProgress::InProgress }

                                let mut state: BufferState = BufferState::Idle; 
                                if busy_full { state = BufferState::BusyFull }
                                else if empty_cmd_in_progress { state = BufferState::BusyEmpty }
                                
                                let _ = tx.send( CmdReceiverState {
                                    discarded,
                                    // progress,
                                    state,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok((rx, task))
    },
    
    // Motor settings
    async fn preset_encoder(&self, position: i32) -> Result<()> {
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
    },
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
    },
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
    },

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
    },
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
    },
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
    },
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
    },
    async fn start_speed_for_degrees2(
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
    },
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
    },
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
    },

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
    },
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
    },

    // Note: Currently the returned channel assumes primary mode is Position. 
    async fn motor_combined_sensor_enable(
        &self,
        // primary_mode: MotorSensorMode,
        speed_delta: u32,
        position_delta: u32,
    // ) -> Result<(broadcast::Receiver<Vec<u8>>, JoinHandle<()>)> {
    ) -> Result<(broadcast::Receiver<(i8, i32)>, JoinHandle<()>)> {
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
        // tokio::time::sleep(Duration::from_millis(100)).await;

        // Step 2: Set up modes
        self.motor_sensor_enable(MotorSensorMode::Speed, speed_delta).await
            ?;
        // tokio::time::sleep(Duration::from_millis(100)).await;    
        // self.motor_sensor_enable(MotorSensorMode::APos, position_delta).await;    // Availablie on TechnicLinear motors, not on InternalTacho (MoveHub)
        self.motor_sensor_enable(MotorSensorMode::Pos, position_delta).await
            ?; // POS available on either
        // tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Step 3: Set up combination
        
        
        // let mut sensor2_mode_nibble: u8;
        let dataset_nibble: u8 = 0x00; // All motor modes have 1 dataset only
        
        // Only pos as primary for now
        // match primary_mode {
        //     MotorSensorMode::Speed => {
        //         sensor0_mode_nibble = 0x10; // Speed
        //         sensor1_mode_nibble = 0x20; // Pos
        //                                     // sensor2_mode_nibble = 0x30; // APos
        //     }
        //     MotorSensorMode::Pos => {
                let sensor0_mode_nibble: u8 = 0x20; // Pos
                let sensor1_mode_nibble: u8 = 0x10; // Speed
                                            // sensor2_mode_nibble = 0x30; // APos
            // }
            // _ => {
            //     sensor0_mode_nibble = 0x00;
            //     sensor1_mode_nibble = 0x00;
            //     // sensor2_mode_nibble = 0x00;
            // }
        // }
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
        // tokio::time::sleep(Duration::from_millis(100)).await;

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
        // tokio::time::sleep(Duration::from_millis(100)).await;

        // Set up channel
        let port_id = self.port();
        let (tx, rx) = broadcast::channel::<(i8, i32)>(64);
        // let (tx, rx) = broadcast::channel::<Vec<u8>>(8);
        match self.get_rx_combined() {
            Ok(mut rx_from_main) => {
                let task = tokio::spawn(async move {
                    let mut position_buffer: i32 = 0;  // Position assumed to be 0 until first update
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        // let _ = tx.send(data.data);

                        // Pos primary
                        // If position changes we always get a speed update even if it has not changed.
                        // If only speed changes then we only get speed => send speed with buffered position. 
                        if data.data.len() == 3 {
                            // tx.send( (data.data[2] as i8, position_buffer) ).expect("Error sending");
                            #[allow(clippy::single_match)]
                            match tx.send( (data.data[2] as i8, position_buffer) ) {
                                Ok(_) => {},
                                Err(_) => { 
                                    // eprintln!("Motor combined error: {:?}", e);
                                }
                            } 
                        }
                        else if data.data.len() == 7 {
                            let mut it = data.data.into_iter().skip(2);
                            let pos = i32::from_le_bytes([
                                it.next().unwrap(),
                                it.next().unwrap(),
                                it.next().unwrap(),
                                it.next().unwrap(),
                            ]);
                            let speed = it.next().unwrap() as i8;
                            position_buffer = pos;
                            #[allow(clippy::single_match)]
                            match tx.send( (speed, pos) ) {
                                Ok(_) => {},
                                Err(_) => { 
                                    // eprintln!("Motor combined error: {:?}", e);
                                }
                            } 
                        }
                        else {
                            eprintln!("Combined mode unexpected length");
                        }

                        // Speed primary
                        // if data.data.len() == 6 {
                        //     let mut it = data.data.into_iter().skip(2);
                        //     let pos = i32::from_le_bytes([
                        //         it.next().unwrap() as u8,
                        //         it.next().unwrap() as u8,
                        //         it.next().unwrap() as u8,
                        //         it.next().unwrap() as u8,
                        //     ]);
                        //     tx.send((0, pos)).expect("Error sending");
                        // }
                        // else if data.data.len() == 7 {
                        //     let mut it = data.data.into_iter().skip(2);
                        //     let speed = it.next().unwrap() as i8;
                        //     let pos = i32::from_le_bytes([
                        //         it.next().unwrap() as u8,
                        //         it.next().unwrap() as u8,
                        //         it.next().unwrap() as u8,
                        //         it.next().unwrap() as u8,
                        //     ]);
                        //     tx.send( (speed, pos) ).expect("Error sending");
                        // }
                        // else {
                        //     eprintln!("Combined mode unexpected length");
                        // }
                        

                        // let converted_data = data.data.into_iter().map(|x| x as i8).collect();
                        // tx.send(converted_data);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError(String::from("Something went wrong"))),
        }
    }

]);
