/// Support for
/// https://rebrickable.com/parts/26912/sensor-color-and-distance-powered-up-2-x-4-x-2/
use async_trait::async_trait;
use btleplug::{api::Characteristic, platform::Peripheral};
use core::fmt::Debug;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use std::sync::Arc;

use super::modes;
use super::modes::VisionSensor as visionmode;
use crate::error::Result;
use crate::notifications::{
    CompletionInfo, InputSetupSingle, NotificationMessage,
    PortOutputCommandFormat, PortOutputSubcommand, PortValueSingleFormat,
    StartupInfo, WriteDirectModeDataPayload,
};
// use crate::consts::Color;

// #[macro_use]
// use crate::notifications::macros::*;

#[derive(Debug, Copy, Clone)]
pub enum DetectedColor {
    NoObject = -1,
    Black = 0,
    Color1 = 1,
    Color2 = 2,
    Blue = 3,
    Color4 = 4,
    Green = 5,
    Color6 = 6,
    Yellow = 7,
    Color8 = 8,
    Red = 9,
    White = 10,
}
pub enum OutputColor {
    Off = 0,
    Blue = 3,
    Green = 5,
    Red = 9,
    White = 10,
}

#[async_trait]
pub trait VisionSensor: Debug + Send + Sync {
    fn vison_sensor_single_enable(
        &self,
        mode: u8,
        delta: u32,
    ) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode,
                delta,
                notification_enabled: true,
            });
        self.commit(msg)
    }

    fn visionsensor_color(
        &self,
    ) -> Result<(broadcast::Receiver<DetectedColor>, JoinHandle<()>)> {
        self.vison_sensor_single_enable(visionmode::COLOR, 1)?;
        let port_id = self.port();
        // Set up channel
        let (tx, rx) = broadcast::channel::<DetectedColor>(8);
        let mut rx_from_main = self
            .get_rx()
            .expect("Single value sender not in device cache");
        let task = tokio::spawn(async move {
            while let Ok(msg) = rx_from_main.recv().await {
                if msg.port_id == port_id {
                    match msg.data[0] {
                        -1 => {
                            let _ = tx.send(DetectedColor::NoObject);
                        }
                        0 => {
                            let _ = tx.send(DetectedColor::Black);
                        }
                        1 => {
                            let _ = tx.send(DetectedColor::Color1);
                        }
                        2 => {
                            let _ = tx.send(DetectedColor::Color2);
                        }
                        3 => {
                            let _ = tx.send(DetectedColor::Blue);
                        }
                        4 => {
                            let _ = tx.send(DetectedColor::Color4);
                        }
                        5 => {
                            let _ = tx.send(DetectedColor::Green);
                        }
                        6 => {
                            let _ = tx.send(DetectedColor::Color6);
                        }
                        7 => {
                            let _ = tx.send(DetectedColor::Yellow);
                        }
                        8 => {
                            let _ = tx.send(DetectedColor::Color8);
                        }
                        9 => {
                            let _ = tx.send(DetectedColor::Red);
                        }
                        10 => {
                            let _ = tx.send(DetectedColor::White);
                        }
                        _ => (),
                    }
                }
            }
        });

        Ok((rx, task))
    }

    // Just setting output mode turns the light off, which may be useful
    fn visionsensor_light_output_mode(&self) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: modes::VisionSensor::COL_O,
                delta: 1,
                notification_enabled: true,
            });
        self.commit(msg)
    }
    // Output colors are limited to R, G, B and W (all three)
    fn visionsensor_set_color(&self, color: OutputColor) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetVisionSensorColor(color as i8),
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg)
    }

    /// Device trait boilerplate
    fn port(&self) -> u8;
    fn tokens(&self) -> (Arc<Peripheral>, Arc<Characteristic>);
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn check(&self) -> Result<()>;
    fn commit(&self, msg: NotificationMessage) -> Result<()> {
        match crate::hubs::send(self.tokens(), msg) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
