use std::collections::BTreeMap;

/// Definition for the Remote Control
use super::*;

#[derive(Debug, )]
pub struct GenericHub {
    peripheral: Peripheral,
    lpf_characteristic: Characteristic,
    properties: HubProperties,
    pub connected_io: BTreeMap<u8, IoDevice>,
    pub kind: crate::consts::HubType
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
    fn properties(&self) -> &HubProperties { &self.properties }
    fn characteristic(&self) -> &Characteristic { &self.lpf_characteristic }
    fn peripheral(&self) -> &Peripheral { &self.peripheral }
    fn connected_io(&mut self) -> &mut BTreeMap<u8, IoDevice> { &mut self.connected_io }
    fn kind(&self) -> HubType { self.kind } 

    fn attach_io(&mut self, device_to_insert: IoDevice) -> Result<()> {
        self.connected_io.insert(device_to_insert.port, device_to_insert );
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

    // Deprecated. Some earlier examples uses this, new examples should 
    // use enable_from_port or enable_from_kind
    async fn port(&self, port_id: Port) -> Result<Box<dyn Device>> {
        let port =
            *self.properties.port_map.get(&port_id).ok_or_else(|| {
                crate::Error::NoneError(format!(
                    "Port type `{port_id:?}` not supported"
                ))
            })?;
        Ok(match port_id {
            Port::HubLed => Box::new(devices::HubLED::new(
                self.peripheral.clone(),
                self.lpf_characteristic.clone(),
                port,
            )),
            Port::A | Port::B  => {
                Box::new(devices::RemoteButtons::new(
                    self.peripheral.clone(),
                    self.lpf_characteristic.clone(),
                    port_id,
                    port,
                ))
            }
            _ => todo!(),
        })
    }

    //TODO: Put actual port_id / kind in error msgs
    async fn get_from_port(&self, port_id: u8) -> Result<IoDevice> {
        match self.connected_io.get(&port_id) {
            Some(connected_device) => { Ok(connected_device.clone()) }
            None => { Err(Error::HubError(String::from("No device on port {port_id}"))) }
        }
    }
    async fn get_from_kind(&self, kind: IoTypeId) -> Result<IoDevice> {
        let mut matches: Vec<&IoDevice> = Vec::new();
        for val in self.connected_io.values() {
            match val.kind {
                kind => { matches.push(val) }
                _ => ()
            }
        }
        match matches.len() {
            0 => {
                Err(Error::NoneError(String::from("No device of kind {kind}")))   
            }
            1 =>  {
                let device_ref = *matches.first().unwrap();
                Ok(device_ref.clone()) 
            }
            _ => { 
                Err(Error::HubError(String::from("Found {kind} on {list of ports}, use enable_from_port"))) 
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

        let mut port_map = PortMap::with_capacity(10);
        port_map.insert(Port::A, 0x0);
        port_map.insert(Port::B, 0x1);
        port_map.insert(Port::HubLed, 0x34);
        port_map.insert(Port::VoltageSensor, 0x3b);
        port_map.insert(Port::Rssi, 0x3c);

        let properties = HubProperties {
            mac_address: props.address.to_string(),
            name: props.local_name.unwrap_or_default(),
            rssi: props.tx_power_level.unwrap_or_default(),
            port_map,
            ..Default::default()
        };
        

        Ok(Self {
            peripheral,
            lpf_characteristic,
            properties,
            connected_io: Default::default(),
            kind
            // stream: peripheral().notifications().await?
        })
    }


}