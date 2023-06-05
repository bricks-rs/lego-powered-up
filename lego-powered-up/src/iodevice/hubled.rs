/// Support for the RGB light in various hubs. Tested with:
/// https://rebrickable.com/parts/85824/hub-powered-up-4-port-technic-control-screw-opening/
/// https://rebrickable.com/sets/88006-1/move-hub/
/// https://rebrickable.com/parts/28739/control-unit-powered-up/

use async_trait::async_trait;
use core::fmt::Debug;
use btleplug::{ api::{Characteristic}, platform::Peripheral };

pub use crate::consts::Color;
use crate::{Result};
use crate::notifications::NotificationMessage;
use crate::notifications::InputSetupSingle;
use crate::notifications::PortOutputSubcommand;
use crate::notifications::PortOutputCommandFormat;
use crate::notifications::WriteDirectModeDataPayload;
use crate::notifications::StartupInfo;
use crate::notifications::CompletionInfo;


#[async_trait]
pub trait HubLed: Debug + Send + Sync {
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
    
    /// Device trait boilerplate
    fn port(&self) -> u8;
    fn tokens(&self) -> (&Peripheral, &Characteristic);
    fn check(&self) -> Result<()>;
    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
        match crate::hubs::send(self.tokens(), msg).await { 
            Ok(()) => Ok(()),
            Err(e)  => { Err(e) }
        }
    }

}

pub enum HubLedMode {
    /// Colour may be set to one of a number of specific named colours
    Colour = 0x0,
    /// Colour may be set to any 12-bit RGB value
    Rgb = 0x01,
    
}
