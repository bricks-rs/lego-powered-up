/// Support for the RGB light in various hubs. Tested with:
/// https://rebrickable.com/parts/85824/hub-powered-up-4-port-technic-control-screw-opening/
/// https://rebrickable.com/sets/88006-1/move-hub/
/// https://rebrickable.com/parts/28739/control-unit-powered-up/
use async_trait::async_trait;
use core::fmt::Debug;

use crate::device_trait;
use super::Basic;
pub use crate::consts::Color;
use crate::notifications::{StartupInfo, CompletionInfo};
use crate::notifications::PortOutputSubcommand;
use crate::notifications::WriteDirectModeDataPayload;
use crate::Result;

#[derive(Debug, Copy, Clone)]
pub enum HubLedMode {
    /// Colour may be set to one of a number of specific named colours
    Colour = 0x0,
    /// Colour may be set to any 12-bit RGB value
    Rgb = 0x01,
}

device_trait!(HubLed, [ 
    async fn set_hubled_mode(&self, mode: HubLedMode) -> Result<()> {
        self.check()?;
        self.device_mode(mode as u8, 1, true).await
    },

    async fn set_hubled_rgb(&self, rgb: &[u8; 3]) -> Result<()> {
        // self.check()?;  // Possible performance hit?
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetHubRgb {
                red: rgb[0],
                green: rgb[1],
                blue: rgb[2],
            },
        );
        self.device_command(subcommand, StartupInfo::ExecuteImmediately, CompletionInfo::NoAction).await
    },

    async fn set_hubled_color(&self, color: Color) -> Result<()> {
        self.check()?;
        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetHubColor(color as i8),
        );
        self.device_command(subcommand, StartupInfo::ExecuteImmediately, CompletionInfo::NoAction).await
    }
]);
