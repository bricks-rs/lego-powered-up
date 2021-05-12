use crate::Hub;
use anyhow::Result;
use btleplug::api::Peripheral;

pub struct TechnicHub<P: Peripheral> {
    pub peripheral: P,
}

impl<P: Peripheral> Hub for TechnicHub<P> {
    fn name(&self) -> String {
        self.peripheral.properties().local_name.unwrap_or_default()
    }

    fn disconnect(&self) -> Result<()> {
        if self.is_connected() {
            self.peripheral.disconnect()?;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.peripheral.is_connected()
    }
}
