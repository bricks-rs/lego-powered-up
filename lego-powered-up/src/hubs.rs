// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Specific implementations for each of the supported hubs.

#![allow(unused)]

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use btleplug::api::ValueNotification;

use std::collections::{HashMap, BTreeMap};
use std::fmt::Debug;

use crate::{PoweredUp, HubFilter, devices::Device, error::Error};
use crate::notifications::NotificationMessage;
use crate::notifications::NetworkCommand::ConnectionRequest;
use crate::notifications::*;
use crate::consts::*;
use devices::iodevice::IoDevice;



use crate::devices::{self, };
use crate::error::{OptionContext, Result};
use crate::notifications::{ModeInformationRequest, ModeInformationType,
     InformationRequest, InformationType,  
     HubAction, HubActionRequest, InputSetupSingle, PortModeInformationType};
use devices::IoTypeId;
use devices::iodevice::*;

use futures::stream::{StreamExt, FuturesUnordered, Stream};
use futures::{future, select};
use core::pin::Pin;
use std::sync::{Arc};
use tokio::sync::Mutex;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

/// Trait describing a generic hub.
#[async_trait::async_trait]
pub trait Hub: Debug + Send + Sync {
    // fn cached_name(&self) -> String;

    async fn name(&self) -> Result<String>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> Result<bool>;
    // The init function cannot be a trait method until we have GAT :(
    //fn init(peripheral: P);
    fn properties(&self) -> &HubProperties;
    fn peripheral(&self) -> &Peripheral;
    fn characteristic(&self) -> &Characteristic;
    fn kind(&self) -> HubType;
    fn connected_io(&mut self) -> &mut BTreeMap<u8, IoDevice>;
    fn channels(&mut self) -> &mut crate::hubs::generic_hub::Channels;

    // Port information
    async fn request_port_info(&self, port_id: u8, infotype: InformationType) -> Result<()> {
        let msg =
        NotificationMessage::PortInformationRequest(InformationRequest {
            port_id,
            information_type: infotype,
        });
        self.send(msg).await
    }
    async fn req_mode_info(&self, port_id: u8, mode: u8, infotype: ModeInformationType) -> Result<()> {
        let msg =
        NotificationMessage::PortModeInformationRequest(ModeInformationRequest {
            port_id,
            mode,
            information_type: infotype,
        });
        self.send(msg).await
    }

    async fn set_port_mode(&self, port_id: u8, mode: u8, delta: u32, notification_enabled: bool) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id,
                mode, 
                delta,
                notification_enabled,
            });
        self.send(mode_set_msg).await
    }

    async fn hub_action(&self, action_type: HubAction) -> Result<()> {
        let msg =
        NotificationMessage::HubActions(HubActionRequest {
            action_type,
        });
        self.send(msg).await
    }
   
    async fn send(&self, msg: NotificationMessage) -> Result<()> {
        let buf = msg.serialise();
        self.peripheral()
            .write(self.characteristic(), &buf, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }

    fn attach_io(&mut self, device_to_insert: IoDevice) -> Result<()>;

    // async fn port_map(&self) -> &PortMap {
    //     &self.properties().await.port_map
    // }

    // cannot provide a default implementation without access to the
    // Peripheral trait from here
    async fn send_raw(&self, msg: &[u8]) -> Result<()>;

    // fn send(&self, msg: NotificationMessage) -> Result<()>;

    async fn subscribe(&self, char: Characteristic) -> Result<()>;

    /// Ideally the vec should be sorted somehow
    // async fn attached_io(&self) -> Vec<IoDevice>;

    // fn process_io_event(&mut self, _evt: AttachedIo);

    async fn port(&self, port_id: Port) -> Result<Box<dyn Device>>;             //Deprecated

    async fn io_from_port(&self, port_id: u8) -> Result<IoDevice>;   
    async fn io_from_kind(&self, kind: IoTypeId) -> Result<IoDevice>;   

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
    AB, // Move Hub
    HubLed,
    CurrentSensor,
    VoltageSensor,
    Accelerometer,
    GyroSensor,
    TiltSensor,
    GestureSensor,
    TemperatureSensor1,
    TemperatureSensor2,
    InternalMotor,
    Rssi,
    RemoteA,
    RemoteB,
    Virtual(u8),
    Deprecated         // Port enum depreacated, have this for backwards comp
}

impl Port {
    /// Returns the ID of the port
    pub fn id(&self) -> u8 {
        todo!()
    }
}

/// Struct representing a device connected to a port
// #[derive(Debug, Clone)]
// pub struct ConnectedIo {
//     /// Name/type of device
//     pub port: Port,
//     /// Internal numeric ID of the device
//     pub port_id: u8,
//     /// Device firmware revision
//     pub fw_rev: VersionNumber,
//     /// Device hardware revision
//     pub hw_rev: VersionNumber,
// }
#[derive(Debug, Clone)]
pub struct ConnectedIo {        //deprecated
    /// Name/type of device
    pub io_type_id: IoTypeId,
    /// Internal numeric ID of the device
    pub port_id: u8,
}


// pub mod technic_hub;
// pub mod remote;
// pub mod move_hub;
pub mod generic_hub;
pub mod io_event;

pub async fn send(p: Peripheral, c: Characteristic, msg: NotificationMessage) -> Result<()> {
    let buf = msg.serialise();
        p.write(&c, &buf, WriteType::WithoutResponse)
        .await?;
    Ok(())
}
