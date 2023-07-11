/// Support for 22168 Light Unit, LED, with Cable, Powered Up
/// https://rebrickable.com/parts/22168/light-unit-led-with-cable-powered-up/
///
/// Needs mode information about this unit to complete

use async_trait::async_trait;
use core::fmt::Debug;

use crate::Result;
use crate::device_trait;
use super::Basic;

device_trait!(HeadLight, []);

