use crate::notifications::NotificationMessage;
use crate::Hub;
use crate::{
    consts::blecharacteristic,
    notifications::{AttachedIo, IoAttachEvent, VersionNumber},
};
use anyhow::{Context, Result};
use btleplug::api::{Characteristic, Peripheral, WriteType};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HubProperties {
    pub name: String,
    pub fw_version: String,
    pub hw_version: String,
    pub mac_address: String,
    pub battery_level: usize,
    pub rssi: i8,
    pub port_map: PortMap,
}

pub type PortMap = HashMap<Port, u8>;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Port {
    A,
    B,
    C,
    D,
    HubLed,
    CurrentSensor,
    VoltageSensor,
    Accelerometer,
    GyroSensor,
    TiltSensor,
    GestureSensor,
    Virtual(u8),
}

impl Port {
    pub fn id(&self) -> u8 {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct ConnectedIo {
    pub port: Port,
    pub port_id: u8,
    pub fw_rev: VersionNumber,
    pub hw_rev: VersionNumber,
}

pub struct TechnicHub<P: Peripheral> {
    peripheral: P,
    lpf_characteristic: Characteristic,
    properties: HubProperties,
    connected_io: HashMap<u8, ConnectedIo>,
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

    fn properties(&self) -> &HubProperties {
        &self.properties
    }

    fn send_raw(&self, msg: &[u8]) -> Result<()> {
        let write_type = WriteType::WithoutResponse;
        Ok(self
            .peripheral
            .write(&self.lpf_characteristic, &msg, write_type)?)
    }

    fn send(&self, msg: NotificationMessage) -> Result<()> {
        let msg = msg.serialise();
        self.send_raw(&msg)?;
        Ok(())
    }

    fn subscribe(&self, char: Characteristic) -> Result<()> {
        Ok(self.peripheral.subscribe(&char)?)
    }

    fn attached_io(&self) -> Vec<ConnectedIo> {
        let mut ret = Vec::with_capacity(self.connected_io.len());
        for (_k, v) in self.connected_io.iter() {
            ret.push(v.clone());
        }

        ret.sort_by_key(|x| x.port_id);

        ret
    }

    fn process_io_event(&mut self, evt: AttachedIo) {
        match evt.event {
            IoAttachEvent::AttachedIo { hw_rev, fw_rev } => {
                if let Some(port) = self.port_from_id(evt.port) {
                    let io = ConnectedIo {
                        port_id: evt.port,
                        port,
                        fw_rev,
                        hw_rev,
                    };
                    self.connected_io.insert(evt.port, io);
                }
            }
            IoAttachEvent::DetachedIo { io_type_id: _ } => {}
            IoAttachEvent::AttachedVirtualIo {
                port_a: _,
                port_b: _,
            } => {}
        }
    }
}

impl<P: Peripheral> TechnicHub<P> {
    pub fn init(peripheral: P, chars: Vec<Characteristic>) -> Result<Self> {
        // Peripheral is already connected before we get here

        println!("\n\nCHARACTERISTICS:\n\n{:?}\n\n", chars);
        let lpf_characteristic = chars
            .iter()
            .find(|c| c.uuid == *blecharacteristic::LPF2_ALL)
            .context("Device does not advertise LPF2_ALL characteristic")?
            .clone();

        let props = peripheral.properties();

        let mut port_map = PortMap::with_capacity(10);
        port_map.insert(Port::A, 0);
        port_map.insert(Port::B, 1);
        port_map.insert(Port::C, 2);
        port_map.insert(Port::D, 3);
        port_map.insert(Port::HubLed, 50);
        port_map.insert(Port::CurrentSensor, 59);
        port_map.insert(Port::VoltageSensor, 60);
        port_map.insert(Port::Accelerometer, 97);
        port_map.insert(Port::GyroSensor, 98);
        port_map.insert(Port::TiltSensor, 99);

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

    fn port_from_id(&self, port_id: u8) -> Option<Port> {
        for (k, v) in self.port_map().iter() {
            if *v == port_id {
                return Some(*k);
            }
        }
        None
    }
}
