// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Specific implementations for each of the supported hubs.

use crate::devices::{self, Device};
use crate::error::{OptionContext, Result};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::collections::HashMap;

/// Trait describing a generic hub.
#[async_trait::async_trait]
pub trait Hub {
    async fn name(&self) -> Result<String>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> Result<bool>;
    // The init function cannot be a trait method until we have GAT :(
    //fn init(peripheral: P);
    async fn properties(&self) -> &HubProperties;

    // async fn port_map(&self) -> &PortMap {
    //     &self.properties().await.port_map
    // }

    // cannot provide a default implementation without access to the
    // Peripheral trait from here
    async fn send_raw(&self, msg: &[u8]) -> Result<()>;

    // fn send(&self, msg: NotificationMessage) -> Result<()>;

    async fn subscribe(&self, char: Characteristic) -> Result<()>;

    /// Ideally the vec should be sorted somehow
    async fn attached_io(&self) -> Vec<ConnectedIo>;

    // fn process_io_event(&mut self, _evt: AttachedIo);

    async fn port(&self, port_id: Port) -> Result<Box<dyn Device>>;
}

pub type VersionNumber = u8;

/// Propeties of a hub
#[derive(Debug, Default)]
pub struct HubProperties {
    /// Friendly name, set via the PoweredUp or Control+ apps
    pub name: String,
    /// Firmware revision
    pub fw_version: String,
    /// Hardware revision
    pub hw_version: String,
    /// BLE MAC address
    pub mac_address: String,
    /// Battery level
    pub battery_level: usize,
    /// BLE signal strength
    pub rssi: i16,
    /// Mapping from port type to port ID. Internally (to the hub) each
    /// port has a hardcoded identifier
    pub port_map: PortMap,
}

pub type PortMap = HashMap<Port, u8>;

/// Ports supported by any hub
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Port {
    /// Motor A
    A,
    /// Motor B
    B,
    /// Motor C
    C,
    /// Motor D
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
    /// Returns the ID of the port
    pub fn id(&self) -> u8 {
        todo!()
    }
}

/// Struct representing a device connected to a port
#[derive(Debug, Clone)]
pub struct ConnectedIo {
    /// Name/type of device
    pub port: Port,
    /// Internal numeric ID of the device
    pub port_id: u8,
    /// Device firmware revision
    pub fw_rev: VersionNumber,
    /// Device hardware revision
    pub hw_rev: VersionNumber,
}



/// Definition for the TechnicMediumHub
pub struct TechnicHub {
    peripheral: Peripheral,
    lpf_characteristic: Characteristic,
    properties: HubProperties,
    connected_io: HashMap<u8, ConnectedIo>,
}

#[async_trait::async_trait]
impl Hub for TechnicHub {
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

    async fn properties(&self) -> &HubProperties {
        &self.properties
    }

    async fn send_raw(&self, msg: &[u8]) -> Result<()> {
        let write_type = WriteType::WithoutResponse;
        Ok(self
            .peripheral
            .write(&self.lpf_characteristic, msg, write_type)
            .await?)
    }

    // fn send(&self, msg: NotificationMessage) -> Result<()> {
    //     let msg = msg.serialise();
    //     self.send_raw(&msg)?;
    //     Ok(())
    // }

    async fn subscribe(&self, char: Characteristic) -> Result<()> {
        Ok(self.peripheral.subscribe(&char).await?)
    }

    async fn attached_io(&self) -> Vec<ConnectedIo> {
        let mut ret = Vec::with_capacity(self.connected_io.len());
        for (_k, v) in self.connected_io.iter() {
            ret.push(v.clone());
        }

        ret.sort_by_key(|x| x.port_id);

        ret
    }

    // fn process_io_event(&mut self, evt: AttachedIo) {
    //     match evt.event {
    //         IoAttachEvent::AttachedIo { hw_rev, fw_rev } => {
    //             if let Some(port) = self.port_from_id(evt.port) {
    //                 let io = ConnectedIo {
    //                     port_id: evt.port,
    //                     port,
    //                     fw_rev,
    //                     hw_rev,
    //                 };
    //                 self.connected_io.insert(evt.port, io);
    //             }
    //         }
    //         IoAttachEvent::DetachedIo { io_type_id: _ } => {}
    //         IoAttachEvent::AttachedVirtualIo {
    //             port_a: _,
    //             port_b: _,
    //         } => {}
    //     }
    // }

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

impl TechnicHub {
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

    // async fn port_from_id(&self, _port_id: u8) -> Option<Port> {
    // for (k, v) in self.port_map().await.iter() {
    //     if *v == port_id {
    //         return Some(*k);
    //     }
    // }
    // None
    // }
}

/// Definition for the TechnicMediumHub
pub struct RemoteControl {
    peripheral: Peripheral,
    lpf_characteristic: Characteristic,
    properties: HubProperties,
    connected_io: HashMap<u8, ConnectedIo>,
}

#[async_trait::async_trait]
impl Hub for RemoteControl {
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

    async fn properties(&self) -> &HubProperties {
        &self.properties
    }

    async fn send_raw(&self, msg: &[u8]) -> Result<()> {
        let write_type = WriteType::WithoutResponse;
        Ok(self
            .peripheral
            .write(&self.lpf_characteristic, msg, write_type)
            .await?)
    }

    // fn send(&self, msg: NotificationMessage) -> Result<()> {
    //     let msg = msg.serialise();
    //     self.send_raw(&msg)?;
    //     Ok(())
    // }

    async fn subscribe(&self, char: Characteristic) -> Result<()> {
        Ok(self.peripheral.subscribe(&char).await?)
    }

    async fn attached_io(&self) -> Vec<ConnectedIo> {
        let mut ret = Vec::with_capacity(self.connected_io.len());
        for (_k, v) in self.connected_io.iter() {
            ret.push(v.clone());
        }

        ret.sort_by_key(|x| x.port_id);

        ret
    }

    // fn process_io_event(&mut self, evt: AttachedIo) {
    //     match evt.event {
    //         IoAttachEvent::AttachedIo { hw_rev, fw_rev } => {
    //             if let Some(port) = self.port_from_id(evt.port) {
    //                 let io = ConnectedIo {
    //                     port_id: evt.port,
    //                     port,
    //                     fw_rev,
    //                     hw_rev,
    //                 };
    //                 self.connected_io.insert(evt.port, io);
    //             }
    //         }
    //         IoAttachEvent::DetachedIo { io_type_id: _ } => {}
    //         IoAttachEvent::AttachedVirtualIo {
    //             port_a: _,
    //             port_b: _,
    //         } => {}
    //     }
    // }

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

impl RemoteControl {
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
        port_map.insert(Port::A, 0);
        port_map.insert(Port::B, 1);
        port_map.insert(Port::HubLed, 50);
        // port_map.insert(Port::CurrentSensor, 59);
        // port_map.insert(Port::VoltageSensor, 60);
        // port_map.insert(Port::Accelerometer, 97);
        // port_map.insert(Port::GyroSensor, 98);
        // port_map.insert(Port::TiltSensor, 99);

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

    // async fn port_from_id(&self, _port_id: u8) -> Option<Port> {
    // for (k, v) in self.port_map().await.iter() {
    //     if *v == port_id {
    //         return Some(*k);
    //     }
    // }
    // None
    // }
}
