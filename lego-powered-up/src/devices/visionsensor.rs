use async_trait::async_trait;
use core::fmt::Debug;

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::IoTypeId;
use crate::devices::modes;
use crate::error::{Error, OptionContext, Result};
use crate::notifications::{NotificationMessage, ValueFormatType, PortValueSingleFormat, DatasetType, InputSetupSingle, PortOutputSubcommand, WriteDirectModeDataPayload, PortOutputCommandFormat, StartupInfo, CompletionInfo};
use crate::devices::modes::VisionSensor as visionmode;
use crate::consts::Color;


// #[macro_use]
// use crate::notifications::macros::*;

#[derive(Debug, Copy, Clone)]
pub enum DetectedColor {
    NoObject = -1,
    Black = 0,
    Magenta = 1,
    Color2 = 2,
    Blue = 3,
    Teal = 4,
    Green = 5,
    Color6 = 6,
    Yellow = 7,
    Color8 = 8,
    Red = 9,
    White = 10
}

#[async_trait]
pub trait VisionSensor: Debug + Send + Sync {
    fn port(&self) -> u8;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn check(&self) -> Option<(Peripheral, Characteristic)>;

    async fn vison_sensor_single_enable(&self, mode: u8, delta: u32) -> Result<()> {
        let pc = match self.check() {
            Some(handles) => handles,
            None => return Err(Error::NoneError((String::from("Not a Vision sensor"))))
        };
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        crate::hubs::send(pc.0, pc.1, msg).await
    }

    async fn visionsensor_color(&self) -> Result<(broadcast::Receiver<DetectedColor>, JoinHandle<()> )> {
        self.vison_sensor_single_enable(visionmode::COLOR, 1).await?;
        let port = self.port();
        // Set up channel
        let (tx, mut rx) = broadcast::channel::<DetectedColor>(8);
        let mut rx_from_main = self.get_rx().expect("Single value sender not in device cache");
                let task = tokio::spawn(async move {
                    while let Ok(msg) = rx_from_main.recv().await {
                        match msg.port_id {
                            port => {
                                match msg.data[0] as i8 {
                                    0 => { tx.send(DetectedColor::Black); }
                                    1 => { tx.send(DetectedColor::Magenta); }
                                    2 => { tx.send(DetectedColor::Color2); }
                                    3 => { tx.send(DetectedColor::Blue); }
                                    4 => { tx.send(DetectedColor::Teal); }
                                    5 => { tx.send(DetectedColor::Green); }
                                    6 => { tx.send(DetectedColor::Color6); }
                                    7 => { tx.send(DetectedColor::Yellow); }
                                    8 => { tx.send(DetectedColor::Color8); }
                                    9 => { tx.send(DetectedColor::Red); }
                                    10 => { tx.send(DetectedColor::White); }
                                    _  => ()
                                }
                            }
                            _ => ()                                
                        }
                    }
                });
            
                Ok((rx, task))
    }

    async fn visionsensor_light_mode(&self) -> Result<()> {
        let pc = match self.check() {
            Some(handles) => handles,
            None => return Err(Error::NoneError((String::from("Not a Vision sensor"))))
        };
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: modes::VisionSensor::COL_O,
                delta: 1,
                notification_enabled: true,
            });
        crate::hubs::send(pc.0, pc.1, msg).await
    }


    async fn visionsensor_set_color(&self, color: i8) -> Result<()> {
        let pc = match self.check() {
            Some(handles) => handles,
            None => return Err(Error::NoneError((String::from("Not a Vision sensor"))))
        };
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColorNo(color as i8));

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        crate::hubs::send(pc.0, pc.1, msg).await
    }

}
