use super::*;
/// Generic hub implementation, tested compatible with:
///
/// Technic Medium hub, aka. Control+
/// https://rebrickable.com/parts/85824/hub-powered-up-4-port-technic-control-screw-opening/
/// Move hub, aka. Boost
/// https://rebrickable.com/sets/88006-1/move-hub/
/// Remote control handset
/// https://rebrickable.com/parts/28739/control-unit-powered-up/
///
/// The Spike and Dacta hubs have not been tested, but there's no
/// no obvious reason why they shouldn't work as long as they
/// use the LEGO Wireless Protocol 3.0.00.
use std::sync::Arc;

use std::collections::BTreeMap;
#[derive(Debug)]
pub struct GenericHub {
    peripheral: Peripheral,
    lpf_characteristic: Characteristic,
    properties: HubProperties,
    pub connected_io: BTreeMap<u8, IoDevice>,
    pub kind: crate::consts::HubType,
    pub channels: Channels,

    peripheral2: Arc<Peripheral>,
    lpf_characteristic2: Arc<Characteristic>,
}

#[async_trait::async_trait]
impl Hub for GenericHub {
    async fn name(&self) -> Result<String> {
        Ok(self
            .peripheral
            .properties()
            .await?
            .context("No properties found for hub")?
            .local_name
            .unwrap_or_default())
    }
    fn properties(&self) -> &HubProperties {
        &self.properties
    }
    fn characteristic(&self) -> &Characteristic {
        &self.lpf_characteristic
    }
    fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }
 
    fn peripheral2(&self) -> Arc<Peripheral>{
        self.peripheral2.clone()
    }
    fn characteristic2(&self) -> Arc<Characteristic>{
        self.lpf_characteristic2.clone()
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

    fn attach_io(&mut self, device_to_insert: IoDevice) -> Result<()> {
        self.connected_io
            .insert(device_to_insert.port(), device_to_insert);
        Ok(())
    }
    async fn disconnect(&self) -> Result<()> {
        if self.is_connected().await? {
            self.peripheral.disconnect().await?;
        }
        Ok(())
    }
    async fn is_connected(&self) -> Result<bool> {
        Ok(self.peripheral.is_connected().await?)
    }
    async fn send_raw(&self, msg: &[u8]) -> Result<()> {
        let write_type = WriteType::WithoutResponse;
        Ok(self
            .peripheral
            .write(&self.lpf_characteristic, msg, write_type)
            .await?)
    }
    async fn subscribe(&self, char: Characteristic) -> Result<()> {
        Ok(self.peripheral.subscribe(&char).await?)
    }

    // async fn attached_io(&self) -> Vec<IoDevice> {
    //     let mut ret = Vec::with_capacity(self.connected_io.len());
    //     for (_k, v) in self.connected_io.iter() {
    //         ret.push(v.clone());
    //     }

    //     ret.sort_by_key(|x| x.port);

    //     ret
    // }

    // Deprecated,  use enable_from_port or enable_from_kind
    // async fn port(&self, port_id: Port) -> Result<Box<dyn Device>> {
    //     let port =
    //         *self.properties.port_map.get(&port_id).ok_or_else(|| {
    //             crate::Error::NoneError(format!(
    //                 "Port type `{port_id:?}` not supported"
    //             ))
    //         })?;
    //     Ok(match port_id {
    //         Port::HubLed => Box::new(devices::HubLED::new(
    //             self.peripheral.clone(),
    //             self.lpf_characteristic.clone(),
    //             port,
    //         )),
    //         Port::A | Port::B  => {
    //             Box::new(devices::RemoteButtons::new(
    //                 self.peripheral.clone(),
    //                 self.lpf_characteristic.clone(),
    //                 port_id,
    //                 port,
    //             ))
    //         }
    //         _ => todo!(),
    //     })
    // }

    /// Cache handles held by hub on device so we don't need to lock hub mutex as often    
    fn device_cache(&self, mut d: IoDevice) -> IoDevice {
        // Channels that forward some notification message types
        d.cache_channels((
            self.channels.singlevalue_sender.clone(),
            self.channels.combinedvalue_sender.clone(),
            self.channels.networkcmd_sender.clone(),
        ));

        // BT handles for calling send
        d.cache_tokens((
            Some(self.peripheral().clone()),
            Some(self.characteristic().clone()),
        ));

        d
    }
    fn device_cache2(&self, mut d: IoDevice) -> IoDevice {
        // Channels that forward some notification message types
        d.cache_channels((
            self.channels.singlevalue_sender.clone(),
            self.channels.combinedvalue_sender.clone(),
            self.channels.networkcmd_sender.clone(),
        ));

        // BT handles for calling send
        d.cache_tokens((
            Some(self.peripheral().clone()),
            Some(self.characteristic().clone()),
        ));
        d.cache_tokens2((
            Some(self.peripheral2().clone()),
            Some(self.characteristic2().clone()),
        ));

        d
    }

    fn io_from_port(&self, port_id: u8) -> Result<IoDevice> {
        match self.connected_io.get(&port_id) {
            Some(connected_device) => {
                let mut d = connected_device.clone();
                d = self.device_cache2(d);

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
            1 =>  {
                let device_deref = *found.first().unwrap();
                let mut d = device_deref.clone();
                d = self.device_cache(d);

                Ok(d)
            }
            _ => {
                Err(Error::HubError(format!("Found {:?} {req_kind:?} on {:?}, use io_from_port or io_multi_from_kind",
                    found.len(), found.iter().map(|x|x.port()).collect::<Vec<_>>()) ))
            }
        }
    }
    fn io_multi_from_kind(
        &self,
        req_kind: IoTypeId,
    ) -> Result<Vec<IoDevice>> {
        let found: Vec<IoDevice> = self
            .connected_io
            .values()
            .filter(|&x| *x.kind() == req_kind)
            .cloned()
            .collect();
        match found.len() {
            0 => {
                Err(Error::HubError(format!("No device of kind: {req_kind:?}")))   
            }
            1..=4 =>  {
                Ok(found)
            }
            _ => {
                Err(Error::HubError(format!("Sanity check: > 4 devices of same kind. Found {:?} {req_kind:?} on {:?}", 
                    found.len(), found.iter().map(|x|x.port()).collect::<Vec<_>>()) ))
            }
        }
    }
}

impl GenericHub {
    /// Initialisation method
    pub async fn init(
        peripheral: Peripheral,
        lpf_characteristic: Characteristic,
        kind: crate::consts::HubType,
    ) -> Result<Self> {
        // Peripheral is already connected before we get here

        let props = peripheral
            .properties()
            .await?
            .context("No properties found for hub")?;

        // let mut port_map = PortMap::with_capacity(10);
        // port_map.insert(Port::A, 0x0);
        // port_map.insert(Port::B, 0x1);
        // port_map.insert(Port::HubLed, 0x34);
        // port_map.insert(Port::VoltageSensor, 0x3b);
        // port_map.insert(Port::Rssi, 0x3c);

        let properties = HubProperties {
            mac_address: props.address.to_string(),
            name: props.local_name.unwrap_or_default(),
            rssi: props.tx_power_level.unwrap_or_default(),
            // port_map,
            ..Default::default()
        };
        let p2 = Arc::new(peripheral.clone());
        let c2= Arc::new(lpf_characteristic.clone());

        Ok(Self {
            peripheral,
            lpf_characteristic,
            properties,
            connected_io: Default::default(),
            kind,
            channels: Default::default(),

            peripheral2: p2,
            lpf_characteristic2: c2,
        })
    }
}
