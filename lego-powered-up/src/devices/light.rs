use async_trait::async_trait;
use core::fmt::Debug;

use btleplug::api::{Characteristic, };
use btleplug::platform::Peripheral;

pub use crate::consts::Color;
use crate::devices::modes;
use crate::{Error, Result};
use crate::notifications::NotificationMessage;
use crate::notifications::InputSetupSingle;
use crate::notifications::PortOutputSubcommand;
use crate::notifications::PortOutputCommandFormat;
use crate::notifications::WriteDirectModeDataPayload;
use crate::notifications::StartupInfo;
use crate::notifications::CompletionInfo;

#[async_trait]
pub trait HubLed: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;

    async fn set_port_mode(&self, mode: u8, delta: u32, notification_enabled: bool) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode, 
                delta,
                notification_enabled,
            });
        crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await
    }

    async fn set_hubled_mode(&self, mode: HubLedMode) -> Result<()> {
        // TODO: check
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta: 1,
                notification_enabled: false,
            });
        let p = match self.p() {
            Some(p) => p,
            None => return {
                eprintln!("Command error: Not a Hub LED");
                Err(Error::NoneError(String::from("Not a Hub LED")))
            }
        };
        crate::hubs::send(p, self.c().unwrap(), mode_set_msg).await
    }

    async fn set_hubled_rgb(&self, rgb: &[u8; 3]) -> Result<()> {
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColors {
                red: rgb[0],
                green: rgb[1],
                blue: rgb[2],
            },
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        let p = match self.p() {
            Some(p) => p,
            None => return Err(Error::NoneError((String::from("Not a Hub LED"))))
        };
        crate::hubs::send(p, self.c().unwrap(), msg).await
    }

    async fn set_hubled_color(&self, color: Color) -> Result<()> {
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColorNo(color as i8));

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        let p = match self.p() {
            Some(p) => p,
            None => return Err(Error::NoneError((String::from("Not a Hub LED"))))
        };
        crate::hubs::send(p, self.c().unwrap(), msg).await
    }
}

pub enum HubLedMode {
    /// Colour may be set to one of a number of specific named colours
    Colour = 0x0,
    /// Colour may be set to any 12-bit RGB value
    Rgb = 0x01,
}

#[async_trait]
pub trait HeadLights: Debug + Send + Sync {
}