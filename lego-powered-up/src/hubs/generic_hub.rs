//! Generic hub implementation, tested compatible with:
//!
//! Technic Medium hub, aka. Control+
//! https://rebrickable.com/parts/85824/hub-powered-up-4-port-technic-control-screw-opening/
//! Move hub, aka. Boost
//! https://rebrickable.com/sets/88006-1/move-hub/
//! Remote control handset
//! https://rebrickable.com/parts/28739/control-unit-powered-up/
//!
//! The Spike and Dacta hubs have not been tested, but there's no
//! no obvious reason why they shouldn't work as long as they
//! use the LEGO Wireless Protocol 3.0.00.

use tokio_util::sync::CancellationToken;

use super::*;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct GenericHub {
    properties: HubProperties,
    pub connected_io: BTreeMap<u8, IoDevice>,
    pub kind: crate::consts::HubType,
    pub channels: Channels,
    cancel: CancellationToken,
    tokens: Tokens,
}

#[async_trait::async_trait]
impl Hub for GenericHub {
    async fn name(&self) -> Result<String> {
        Ok(self
            .tokens
            .0 //peripheral
            .properties()
            .await?
            .context("No properties found for hub")?
            .local_name
            .unwrap_or_default())
    }
    fn properties(&self) -> &HubProperties {
        &self.properties
    }
    fn peripheral(&self) -> Arc<Peripheral> {
        Arc::new(self.tokens.0.clone())
    }
    fn characteristic(&self) -> Arc<Characteristic> {
        Arc::new(self.tokens.1.clone())
    }
    fn connected_io(&self) -> &BTreeMap<u8, IoDevice> {
        &self.connected_io
    }
    fn connected_io_mut(&mut self) -> &mut BTreeMap<u8, IoDevice> {
        &mut self.connected_io
    }
    fn kind(&self) -> HubType {
        self.kind
    }
    fn channels(&mut self) -> &mut Channels {
        &mut self.channels
    }

    async fn disconnect(&self) -> Result<()> {
        if self.is_connected().await? {
            self.cancel.cancel();
            self.tokens.0.disconnect().await?;
        }
        Ok(())
    }
    async fn is_connected(&self) -> Result<bool> {
        Ok(self.tokens.0.is_connected().await?)
    }
    async fn shutdown(&self) -> Result<()> {
        self.cancel.cancel();
        self.hub_action(crate::notifications::HubAction::SwitchOffHub)
            .await
    }
    async fn send_raw(&self, msg: &[u8]) -> Result<()> {
        let write_type = WriteType::WithoutResponse;
        Ok(self.tokens.0.write(&self.tokens.1, msg, write_type).await?)
    }
    async fn subscribe(&self, char: Characteristic) -> Result<()> {
        Ok(self.tokens.0.subscribe(&char).await?)
    }
    fn cancel_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }
    fn attach_io(&mut self, io_type_id: IoTypeId, port_id: u8) -> Result<()> {
        let device = IoDevice::new(io_type_id, port_id, self.tokens.clone());
        self.connected_io.insert(port_id, device);

        Ok(())
    }

    /// Cache handles held by hub on device so we don't need to lock hub mutex as often    
    fn device_cache(&self, mut d: IoDevice) -> IoDevice {
        // Channels that forward some notification message types
        d.cache_channels(self.channels.clone());

        d
    }

    fn io_from_port(&self, port_id: u8) -> Result<IoDevice> {
        match self.connected_io.get(&port_id) {
            Some(connected_device) => {
                let mut d = connected_device.clone();
                d = self.device_cache(d);

                Ok(d)
            }
            None => Err(Error::HubError(format!(
                "No device on port: {} ({:#x}) ",
                &port_id, &port_id
            ))),
        }
    }

    fn io_from_kind(&self, req_kind: IoTypeId) -> Result<IoDevice> {
        let found: Vec<&IoDevice> = self
            .connected_io
            .values()
            .filter(|&device| *device.kind() == req_kind)
            .collect();
        match found.len() {
            0 => {
                Err(Error::HubError(format!("No device of kind: {req_kind:?}")))
            }
            1 => {
                let device_deref = *found.first().unwrap();
                let mut d = device_deref.clone();
                d = self.device_cache(d);

                Ok(d)
            }
            _ => Err(Error::HubError(format!(
                concat!(
                    "Found {:?} {:?} on {:?}, use io_from_port ",
                    "or io_multi_from_kind"
                ),
                found.len(),
                req_kind,
                found.iter().map(|x| x.port()).collect::<Vec<_>>()
            ))),
        }
    }
    fn io_multi_from_kind(&self, req_kind: IoTypeId) -> Result<Vec<IoDevice>> {
        let found: Vec<IoDevice> = self
            .connected_io
            .values()
            .filter(|&x| *x.kind() == req_kind)
            .cloned()
            .collect();
        match found.len() {
            0 => Err(Error::HubError(format!(
                "No device of kind: {:?}",
                req_kind
            ))),
            1..=4 => Ok(found),
            _ => Err(Error::HubError(format!(
                concat!(
                    "Sanity check: > 4 devices of same kind. ",
                    "Found {:?} {:?} on {:?}"
                ),
                found.len(),
                req_kind,
                found.iter().map(|x| x.port()).collect::<Vec<_>>()
            ))),
        }
    }
}

impl GenericHub {
    /// Initialisation method
    pub async fn init(
        peripheral: Peripheral,
        lpf_characteristic: Characteristic,
        kind: crate::consts::HubType,
        cancel: CancellationToken,
    ) -> Result<Self> {
        // Peripheral is already connected before we get here

        let props = peripheral
            .properties()
            .await?
            .context("No properties found for hub")?;

        let properties = HubProperties {
            mac_address: props.address.to_string(),
            name: props.local_name.unwrap_or_default(),
            rssi: props.tx_power_level.unwrap_or_default(),
            ..Default::default()
        };

        Ok(Self {
            tokens: Arc::new((peripheral, lpf_characteristic)),
            properties,
            connected_io: Default::default(),
            kind,
            channels: Default::default(),
            cancel,
        })
    }
}
