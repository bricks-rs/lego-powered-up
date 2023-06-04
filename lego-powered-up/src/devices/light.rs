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
    fn port(&self) -> u8;
    fn tokens(&self) -> (&Peripheral, &Characteristic);
    fn check(&self) -> Result<()>;

    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
        let tokens = self.tokens();
        match crate::hubs::send2(tokens.0, tokens.1, msg).await { 
            Ok(()) => Ok(()),
            Err(e)  => { Err(e) }
        }
    }

    async fn set_hubled_mode(&self, mode: HubLedMode) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta: 1,
                notification_enabled: false,
            });
        self.commit(msg).await
    }

    async fn set_hubled_rgb(&self, rgb: &[u8; 3]) -> Result<()> {
        // self.check()?;  // Possible performance hit?
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
        self.commit(msg).await
    }

    async fn set_hubled_color(&self, color: Color) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColorNo(color as i8));

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.commit(msg).await
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