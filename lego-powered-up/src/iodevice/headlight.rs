/// Support for 22168 Light Unit, LED, with Cable, Powered Up
/// https://rebrickable.com/parts/22168/light-unit-led-with-cable-powered-up/
///
/// Needs mode information about this unit to complete

use async_trait::async_trait;
use core::fmt::Debug;

use crate::hubs::Tokens;
use crate::notifications::NotificationMessage;
use crate::Result;
use crate::device_trait;

device_trait!(HeadLight, []);

