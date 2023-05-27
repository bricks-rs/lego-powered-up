#![allow(unused)]

/// Definition for the Move Hub
use crate::notifications::NotificationMessage;
use crate::consts::HubPropertyReference;
use crate::consts::HubPropertyOperation;
use crate::consts::HubPropertyPayload;



use super::*;
#[derive(Debug, )]
pub struct MoveHub {
    peripheral: Peripheral,
    lpf_characteristic: Characteristic,
    properties: HubProperties,
    connected_io: BTreeMap<u8, IoDevice>,
}

#[async_trait::async_trait]
impl Hub for MoveHub {
    async fn name(&self) -> Result<String> {
        Ok(self
            .peripheral
            .properties()
            .await?
            .context("No properties found for hub")?
            .local_name
            .unwrap_or_default())
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
    

    fn properties(&self) -> &HubProperties {
        &self.properties
    }
    fn characteristic(&self) -> &Characteristic {
        &self.lpf_characteristic
    }
    fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    fn attach_io(&mut self, device_to_insert: IoDevice) -> Result<()> {
        self.connected_io.insert(device_to_insert.port, device_to_insert );
        dbg!(&self.connected_io);
        Ok(())

    }

    fn connected_io(&mut self) -> &mut BTreeMap<u8, IoDevice> {
        &mut self.connected_io
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
            Port::A | Port::B | Port::C | Port::D => {
                Box::new(devices::Motor::new(
                    self.peripheral.clone(),
                    self.lpf_characteristic.clone(),
                    port_id,
                    port,
                ))
            }
            _ => todo!(),
        })
    }
}

//     PORT_A = 0x00                
//     PORT_B = 0x01                
//     PORT_C = 0x02
//     PORT_D = 0x03
//     PORT_AB = 0x10               16
//     PORT_LED = 0x32              50
//     PORT_TILT_SENSOR = 0x3A      58
//     PORT_CURRENT = 0x3B          59
//     PORT_VOLTAGE = 0x3C          60
//                                  70


impl MoveHub {
    /// Initialisation method
    pub async fn init(
        peripheral: Peripheral,
        lpf_characteristic: Characteristic,
    ) -> Result<Self> {
        // Peripheral is already connected before we get here

        let props = peripheral
            .properties()
            .await?
            .context("No properties found for hub")?;

        let mut port_map = PortMap::with_capacity(10);
        port_map.insert(Port::A, 0x0);
        port_map.insert(Port::B, 0x1);
        port_map.insert(Port::C, 0x2);
        port_map.insert(Port::D, 0x3);
        port_map.insert(Port::AB, 0x10);
        port_map.insert(Port::HubLed, 0x32);
        port_map.insert(Port::TiltSensor, 0x3a);
        port_map.insert(Port::CurrentSensor, 0x3b);
        port_map.insert(Port::VoltageSensor, 0x3c);
        // port_map.insert(Port::, 0x46);                  // Unknown device on this port. Its 3 modes are named TRIGGER, CANVAS and VAR


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
        })
    }

    fn characteristic(&self) -> &Characteristic {
        &self.lpf_characteristic
    }
    fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }
 

    // async fn get_prop(&mut self, property_ref: HubPropertyReference) -> Result<()> {
    //     use crate::notifications::*;

    //     // let subcommand = PortOutputSubcommand::StartSpeed {
    //     //     speed,
    //     //     max_power,
    //     //     use_acc_profile: true,
    //     //     use_dec_profile: true,
    //     // };

    //     let msg =
    //         NotificationMessage::HubProperties(HubProperty {
    //             property_ref,
    //             operation,
    //             payload,
    //         });
    //     self.send(msg).await
    // }

    async fn send(&mut self, msg: NotificationMessage) -> Result<()> {
        
        let buf = msg.serialise();
        self.peripheral()
            .write(self.characteristic(), &buf, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }

}