/// Representation of an IoDevice 
///     This struct + device traits (mod sensor, motor, et.al.) replaces
///     the previous design in mod devices. It also replaces and extends
///     functionality of ConnectedIo, PortMap and Port.
///     This design is basically flipped around; instead of a single device
///     trait with implementations for specific devices we have a single
///     device struct with traits for specific devices. 
///     This fits because all devices share the same data structure (as
///     defined by the data available from the various information-
///     commands) but differ in functionality.
///    
///     Considerations: 
/// 


use std::collections::{BTreeMap};
use std::fmt;

use btleplug::platform::Peripheral;
use btleplug::api::Characteristic;
use tokio::sync::broadcast;

use crate::IoTypeId;
use crate::notifications::{ValueFormatType, DatasetType, MappingValue, PortValueSingleFormat, PortValueCombinedFormat, NetworkCommand};
use crate::devices::remote::RcDevice;
use crate::devices::sensor::*;
use crate::devices::motor::*;
use crate::devices::light::*;
use crate::devices::visionsensor::*;
use crate::{Error, Result};
use crate::hubs::Channels;

pub mod definition;
use definition::Definition;
type ModeId = u8;

#[derive(Debug, Default, Clone)]
pub struct IoDevice {
    kind: IoTypeId,
    port: u8,
    pub def: definition::Definition,
    pub handles: Handles,
    channels: Channels,
}
#[derive(Debug, Default, Clone)]
pub struct Handles {
   pub p: Option<btleplug::platform::Peripheral>,
   pub c: Option<btleplug::api::Characteristic>,
}

impl IoDevice {
    pub fn kind(&self) -> &IoTypeId { &self.kind }
    pub fn def(&self) -> &Definition { &self.def }
    pub fn port(&self) -> u8 { self.port }
    pub fn handles(&self) -> u8 { self.port }
    pub fn set_channels(&mut self, senders:(
        Option<broadcast::Sender<PortValueSingleFormat>>, 
        Option<broadcast::Sender<PortValueCombinedFormat>>, 
        Option<broadcast::Sender<NetworkCommand>>)) {
        (self.channels.singlevalue_sender,
        self.channels.combinedvalue_sender,
        self.channels.networkcmd_sender ) = senders; 
    }

    pub fn new(kind: IoTypeId, port: u8) -> Self {
        Self {
            kind,
            port,
            def: Definition::new(kind, port),
            handles: Default::default(),
            channels: Default::default(),
        }
    }
    pub fn new_with_handles(kind: IoTypeId, port: u8, 
               peripheral: btleplug::platform::Peripheral, 
               characteristic: btleplug::api::Characteristic ) -> Self {
        let handles = Handles {
            p: Some(peripheral),
            c: Some(characteristic),
        };
        Self {
            kind,
            port,
            def: Definition::new(kind, port),
            handles,
            channels: Default::default(),        }
    }
   
}

//
// Implement device-traits
//
impl GenericSensor for IoDevice {
    fn port(&self) -> u8 { self.port }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (&self.handles.p.as_ref().unwrap(), &self.handles.c.as_ref().unwrap())
    }    
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found"))) 
        }
    }
    // Need better abstraction to check at compile
    fn check(&self, mode: u8, datasettype: DatasetType) -> Result<()> {
        if let Some(pm) = self.def.modes().get(&mode) {
            let vf = pm.value_format;
            // println!("Dataset for call: {:?}  Dataset for device: {:?} Device kind: {:?} ", &datasettype, &vf.dataset_type, &self.kind);
            // dbg!(&self.modes);
            if datasettype == vf.dataset_type { return Ok(()) }
            else { return Err(Error::NoneError(String::from("Incorrect dataset type"))) }
        } else {
            Err(Error::NoneError(String::from("Mode not found")))
        }
    }
}

impl RcDevice for IoDevice {
    fn port(&self) -> u8 { self.port }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (&self.handles.p.as_ref().unwrap(), &self.handles.c.as_ref().unwrap())
    } 
    fn get_rx_pvs(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found"))) 
        }
    }
    fn get_rx_nwc(&self) -> Result<broadcast::Receiver<NetworkCommand>> {
        if let Some(sender) = &self.channels.networkcmd_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found"))) 
        }
    }
    fn check(&self) -> Result<()> {
        match self.kind {
            IoTypeId::RemoteButtons => Ok(()),
            _ => Err(Error::HubError(String::from("Not a remote control device"))),
        } 
    } 
}

impl EncoderMotor for IoDevice {   
    fn port(&self) -> u8 { self.port }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (&self.handles.p.as_ref().unwrap(), &self.handles.c.as_ref().unwrap())
    } 
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found"))) 
        }
    }
    fn get_rx_combined(&self) -> Result<broadcast::Receiver<PortValueCombinedFormat>> {
        if let Some(sender) = &self.channels.combinedvalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found"))) 
        }
    }
    fn check(&self) -> Result<()> {
        match self.kind {
            IoTypeId::TechnicLargeLinearMotor |
            IoTypeId::TechnicXLargeLinearMotor |
            IoTypeId::InternalMotorTacho => Ok(()),
            _ => Err(Error::HubError(String::from("Not an Encoder Motor"))),
        } 
    } 
}

impl HubLed for IoDevice {
    fn port(&self) -> u8 { self.port }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (&self.handles.p.as_ref().unwrap(), &self.handles.c.as_ref().unwrap())
    }

    fn check(&self) -> Result<()> {
        match self.kind {
            IoTypeId::HubLed => Ok(()),
            _ => Err(Error::HubError(String::from("Not a Hub LED device"))),
        } 
    } 
}

impl VisionSensor for IoDevice {
    fn port(&self) -> u8 { self.port }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (&self.handles.p.as_ref().unwrap(), &self.handles.c.as_ref().unwrap())
    }
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found"))) 
        }
    }

    fn check(&self) -> Result<()> {
        match self.kind {
            IoTypeId::VisionSensor => Ok(()),
            _ => Err(Error::HubError(String::from("Not a Vision sensor Motor"))),
        } 
    } 
}