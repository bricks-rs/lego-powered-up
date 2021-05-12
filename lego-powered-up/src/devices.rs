use crate::hubs::Port;
use crate::Hub;
use anyhow::Result;

pub trait Device {
    fn port(&self) -> Port;
}

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

impl Device for HubLED {
    fn port(&self) -> Port {
        Port::HubLed
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

    pub fn set_colour(
        &mut self,
        colour: &[u8; 3],
        hub: &Box<dyn Hub>,
    ) -> Result<()> {
        self.colour = *colour;
        hub.send(Port::HubLed, HubLedMode::Rgb as u8, colour, false)
    }
}
