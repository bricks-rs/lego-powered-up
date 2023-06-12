use crate::notifications::NotificationMessage;
use crate::Result;
/// Support for 22168 Light Unit, LED, with Cable, Powered Up
/// https://rebrickable.com/parts/22168/light-unit-led-with-cable-powered-up/
///
/// Needs mode information about this unit to complete
use async_trait::async_trait;
use btleplug::api::Characteristic;
use btleplug::platform::Peripheral;
use core::fmt::Debug;

#[async_trait]
pub trait HeadLight: Debug + Send + Sync {
    /// Device trait boilerplate
    fn port(&self) -> u8;
    fn tokens(&self) -> (&Peripheral, &Characteristic);
    fn check(&self) -> Result<()>;
    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
        match crate::hubs::send(self.tokens(), msg).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
