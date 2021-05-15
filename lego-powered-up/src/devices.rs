use crate::hubs::Port;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::fmt::Debug;
use std::ops::Deref;

#[async_trait]
pub trait Device: Debug + Send + Sync {
    fn port(&self) -> Port;
    async fn set_colour(&mut self, _colour: &[u8; 3]) -> Result<()> {
        Err(anyhow!("Not implemented for type"))
    }
}

#[derive(Debug, Clone)]
pub struct HubLED {
    colour: [u8; 3],
    mode: HubLedMode,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HubLedMode {
    Colour = 0x0,
    Rgb = 0x01,
}

#[async_trait]
impl Device for HubLED {
    fn port(&self) -> Port {
        Port::HubLed
    }

    async fn set_colour(&mut self, colour: &[u8; 3]) -> Result<()> {
        self.colour = *colour;
        //hub.send(Port::HubLed, HubLedMode::Rgb as u8, colour, false)
        todo!()
    }
}

impl HubLED {
    pub fn new(/*hub: &Box<dyn Hub>*/) -> Self {
        let mode = HubLedMode::Rgb;
        //hub.subscribe(mode);
        Self {
            colour: [0; 3],
            mode,
        }
    }
}
