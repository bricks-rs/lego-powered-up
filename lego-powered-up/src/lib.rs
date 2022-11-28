use btleplug::api::{
    bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _,
    PeripheralProperties, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use num_traits::FromPrimitive;

pub mod consts;
pub mod devices;
pub mod error;
pub mod hubs;
pub mod notifications;

pub use error::{Error, OptionContext, Result};

use consts::{BLEManufacturerData, HubType};
use hubs::Hub;

pub struct PoweredUp {
    adapter: Adapter,
}

impl PoweredUp {
    pub async fn devices() -> Result<Vec<Adapter>> {
        let manager = Manager::new().await?;
        Ok(manager.adapters().await?)
    }

    pub async fn init() -> Result<Self> {
        let manager = Manager::new().await?;
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .next()
            .context("No adapter found")?;
        Self::with_adapter(adapter).await
    }

    pub async fn with_device_index(index: usize) -> Result<Self> {
        let manager = Manager::new().await?;
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .nth(index)
            .context("No adapter found")?;
        Self::with_adapter(adapter).await
    }

    pub async fn with_adapter(adapter: Adapter) -> Result<Self> {
        Ok(Self { adapter })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.adapter.start_scan(ScanFilter::default()).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn find_hub(&mut self) -> Result<Option<DiscoveredHub>> {
        let hubs = self.list_discovered_hubs().await?;
        Ok(hubs.into_iter().next())
    }

    pub async fn list_discovered_hubs(&mut self) -> Result<Vec<DiscoveredHub>> {
        let peripherals = self.adapter.peripherals().await?;
        let mut hubs = Vec::new();
        for peripheral in peripherals {
            let Some(props) = peripheral.properties().await? else{continue;};
            if let Some(hub_type) = identify_hub(&props).await? {
                hubs.push(DiscoveredHub {
                    hub_type,
                    addr: peripheral.id(),
                    name: props
                        .local_name
                        .unwrap_or_else(|| "unknown".to_string()),
                });
            }
        }
        Ok(hubs)
    }

    // todo use notifications event stream
    pub async fn wait_for_hub(&mut self) -> Result<DiscoveredHub> {
        loop {
            let hubs = self.list_discovered_hubs().await?;
            if let Some(hub) = hubs.into_iter().next() {
                return Ok(hub);
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
    // todo use notifications event stream
    pub async fn wait_for_hub_filter(
        &mut self,
        filter: HubFilter,
    ) -> Result<DiscoveredHub> {
        loop {
            let hubs = self.list_discovered_hubs().await?;
            if let Some(hub) = hubs.into_iter().find(|hub| filter.matches(hub))
            {
                return Ok(hub);
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    pub async fn create_hub(
        &mut self,
        hub: &DiscoveredHub,
    ) -> Result<Box<dyn Hub>> {
        todo!()
    }
}

/// Properties by which to filter discovered hubs
#[derive(Debug)]
pub enum HubFilter {
    /// Hub name must match the provided value
    Name(String),
    /// Hub address must match the provided value
    Addr(String),
}

impl HubFilter {
    /// Test whether the discovered hub matches the provided filter mode
    pub fn matches(&self, hub: &DiscoveredHub) -> bool {
        use HubFilter::*;
        match self {
            Name(n) => hub.name == *n,
            Addr(a) => format!("{:?}", hub.addr) == *a,
        }
    }
}

/// Struct describing a discovered hub. This description may be passed
/// to `PoweredUp::create_hub` to initialise a connection.
#[derive(Clone, Debug)]
pub struct DiscoveredHub {
    /// Type of hub, e.g. TechnicMediumHub
    pub hub_type: HubType,
    /// BLE address
    pub addr: PeripheralId,
    /// Friendly name of the hub, as set in the PoweredUp/Control+ apps
    pub name: String,
}

async fn identify_hub(props: &PeripheralProperties) -> Result<Option<HubType>> {
    use HubType::*;

    if props
        .services
        .contains(&consts::bleservice::WEDO2_SMART_HUB)
    {
        return Ok(Some(Wedo2SmartHub));
    } else if props.services.contains(&consts::bleservice::LPF2_HUB) {
        if let Some(manufacturer_id) = props.manufacturer_data.get(&919) {
            // Can't do it with a match because some devices are just manufacturer
            // data while some use other characteristics
            if let Some(m) = BLEManufacturerData::from_u8(manufacturer_id[1]) {
                use BLEManufacturerData::*;
                return Ok(Some(match m {
                    DuploTrainBaseId => DuploTrainBase,
                    HubId => Hub,
                    MarioId => Mario,
                    MoveHubId => MoveHub,
                    RemoteControlId => RemoteControl,
                    TechnicMediumHubId => TechnicMediumHub,
                }));
            }
        }
    }
    Ok(None)
}
