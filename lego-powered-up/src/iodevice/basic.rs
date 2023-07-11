/// The basic bricks of device control.
/// All the other device traits are sugar for these 3 commands.

use async_trait::async_trait;
use core::fmt::Debug;
use tokio::sync::broadcast;

use crate::hubs::Tokens;
use crate::error::Result;
use crate::notifications::{
    CompletionInfo, InputSetupCombined, InputSetupCombinedSubcommand,
    InputSetupSingle, NotificationMessage, PortOutputCommandFormat,
    PortOutputSubcommand, StartupInfo, PortValueSingleFormat,
};

#[async_trait]
pub trait Basic: Debug + Send + Sync {
    fn port(&self) -> u8;
    fn tokens(&self) -> Tokens;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
         match crate::hubs::send(self.tokens(), msg).await {
             Ok(()) => Ok(()),
             Err(e) => Err(e),
         }
    }

    async fn device_mode(
        &self,
        mode: u8,
        delta: u32,
        notification_enabled: bool,
    ) -> Result<()> {
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode,
                delta,
                notification_enabled,
            });
        self.commit(msg).await
    }

    async fn device_mode_combined(
        &self,
        subcommand: InputSetupCombinedSubcommand,
    ) -> Result<()> {
        let msg = NotificationMessage::PortInputFormatSetupCombinedmode(
            InputSetupCombined {
                port_id: self.port(),
                subcommand,
            },
        );
        self.commit(msg).await
    }

    async fn device_command(
        &self,
        subcommand: PortOutputSubcommand,
        startup_info: StartupInfo,
        completion_info: CompletionInfo,
    ) -> Result<()> {
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port(),
                startup_info,
                completion_info,
                subcommand,
            });
        self.commit(msg).await
    }
}
