/// Support for
/// https://rebrickable.com/parts/26912/sensor-color-and-distance-powered-up-2-x-4-x-2/

use async_trait::async_trait;
use core::fmt::Debug;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::device_trait;
use super::Basic;
use super::modes;
use crate::error::Result;
use crate::notifications::{
    CompletionInfo, PortOutputSubcommand,
    StartupInfo, WriteDirectModeDataPayload,
};

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
#[derive(Debug, Copy, Clone)]
pub enum OutputColor {
    Off = 0,
    Blue = 3,
    Green = 5,
    Red = 9,
    White = 10,
}

device_trait!(VisionSensor, [
    async fn visionsensor_color(
        &self,
    ) -> Result<(broadcast::Receiver<DetectedColor>, JoinHandle<()>)> {
        self.device_mode(modes::VisionSensor::COLOR, 1, true).await?;
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
    },

    // Just setting output mode turns the light off, which may be useful
    async fn visionsensor_light_output_mode(&self) -> Result<()> {
        self.check()?;
        self.device_mode(modes::VisionSensor::COL_O, 1, true).await
    },

    // Output colors are limited to R, G, B and W (all three)
    async fn visionsensor_set_color(&self, color: OutputColor) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetVisionSensorColor(color as i8),
        );
        self.device_command(subcommand, StartupInfo::ExecuteImmediately, CompletionInfo::NoAction).await
    }
]);
