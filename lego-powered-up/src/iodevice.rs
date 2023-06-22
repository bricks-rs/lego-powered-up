use btleplug::api::Characteristic;
/// Representation of an IoDevice
// use std::collections::{BTreeMap};
// use std::fmt;
use btleplug::platform::Peripheral;
use tokio::sync::broadcast;
use std::sync::Arc;

use crate::hubs::{Channels, Tokens};
use crate::hubs::Tokens2;
use crate::notifications::{
    DatasetType, NetworkCommand, PortValueCombinedFormat, PortValueSingleFormat,
};
use crate::IoTypeId;
use crate::{Error, Result};
use basic::Basic;
use definition::Definition;
use hubled::HubLed;
use motor::EncoderMotor;
use remote::RcDevice;
use sensor::GenericSensor;
use visionsensor::VisionSensor;

pub mod basic;
pub mod definition;
pub mod headlight;
pub mod hubled;
pub mod modes;
pub mod motor;
pub mod remote;
pub mod sensor;
pub mod visionsensor;

#[derive(Debug, Default, Clone)]
pub struct IoDevice {
    pub def: Definition,
    tokens: Tokens,
    tokens2: Tokens2,
    channels: Channels,
}

impl IoDevice {
    pub fn kind(&self) -> &IoTypeId {
        self.def.kind()
    }
    pub fn def(&self) -> &Definition {
        &self.def
    }
    pub fn port(&self) -> u8 {
        self.def.port()
    }
    // pub fn tokens(&self) -> &Tokens {
    //     &self.tokens
    // }
    pub fn channels(&self) -> &Channels {
        &self.channels
    }
    pub fn new(kind: IoTypeId, port: u8) -> Self {
        Self {
            def: Definition::new(kind, port),
            tokens: Default::default(),
            channels: Default::default(),
            tokens2: Default::default(),
        }
    }
    pub fn cache_tokens(
        &mut self,
        tokens: (Option<Peripheral>, Option<Characteristic>),
    ) {
        (self.tokens.p, self.tokens.c) = tokens;
    }
    pub fn cache_tokens2(
        &mut self,
        tokens: (Option<Arc<Peripheral>>, Option<Arc<Characteristic>>),
    ) {
        (self.tokens2.p, self.tokens2.c) = tokens;
    }
    #[allow(clippy::type_complexity)]
    pub fn cache_channels(
        &mut self,
        senders: (
            Option<broadcast::Sender<PortValueSingleFormat>>,
            Option<broadcast::Sender<PortValueCombinedFormat>>,
            Option<broadcast::Sender<NetworkCommand>>,
        ),
    ) {
        (
            self.channels.singlevalue_sender,
            self.channels.combinedvalue_sender,
            self.channels.networkcmd_sender,
        ) = senders;
    }
}

//
// Implement device-traits
//
impl Basic for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> Result<(&Peripheral, &Characteristic)> {
        // (&self.tokens.p.as_ref().unwrap(), &self.tokens.c.as_ref().unwrap())

        match (&self.tokens.p.as_ref(), &self.tokens.c.as_ref()) {
            (Some(p), Some(c)) => Ok((p, c)),
            _ => {
                Err(Error::NoneError(String::from("Token not in device cache")))
            }
        }
    }
}
impl GenericSensor for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (
            (self.tokens.p.as_ref().unwrap()),
            (self.tokens.c.as_ref().unwrap()),
        )
    }
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found")))
        }
    }
    fn check(&self, mode: u8, datasettype: DatasetType) -> Result<()> {
        if let Some(pm) = self.def.modes().get(&mode) {
            let vf = pm.value_format;
            // println!("Dataset for call: {:?}  Dataset for device: {:?} Device kind: {:?} ", &datasettype, &vf.dataset_type, &self.kind);
            // dbg!(&self.modes);
            if datasettype == vf.dataset_type {
                Ok(())
            } else {
                Err(Error::NoneError(String::from("Incorrect dataset type")))
            }
        } else {
            Err(Error::NoneError(String::from("Mode not found")))
        }
    }
}

impl RcDevice for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (
            (self.tokens.p.as_ref().unwrap()),
            (self.tokens.c.as_ref().unwrap()),
        )
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
        match self.def.kind() {
            IoTypeId::RemoteButtons => Ok(()),
            _ => Err(Error::HubError(String::from(
                "Not a remote control device",
            ))),
        }
    }
}

impl EncoderMotor for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (
            (self.tokens.p.as_ref().expect("No peripheral")),
            (self.tokens.c.as_ref().expect("No characteristic")),
        )
    }
    fn tokens2(&self) -> (Arc<Peripheral>, Arc<Characteristic>) {
        (
            (self.tokens2.p.as_ref().expect("No peripheral").clone()),
            (self.tokens2.c.as_ref().expect("No characteristic").clone()),
        )
    }
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found")))
        }
    }
    fn get_rx_combined(
        &self,
    ) -> Result<broadcast::Receiver<PortValueCombinedFormat>> {
        if let Some(sender) = &self.channels.combinedvalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found")))
        }
    }
    fn check(&self) -> Result<()> {
        match self.def.kind() {
            IoTypeId::TechnicLargeLinearMotor
            | IoTypeId::TechnicXLargeLinearMotor
            | IoTypeId::InternalMotorTacho => Ok(()),
            _ => Err(Error::HubError(String::from("Not an Encoder Motor"))),
        }
    }
}

impl HubLed for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (
            (self.tokens.p.as_ref().unwrap()),
            (self.tokens.c.as_ref().unwrap()),
        )
    }
    fn check(&self) -> Result<()> {
        match self.def.kind() {
            IoTypeId::HubLed => Ok(()),
            _ => Err(Error::HubError(String::from("Not a Hub LED device"))),
        }
    }
}

impl VisionSensor for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> (&Peripheral, &Characteristic) {
        (
            (self.tokens.p.as_ref().unwrap()),
            (self.tokens.c.as_ref().unwrap()),
        )
    }
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found")))
        }
    }
    fn check(&self) -> Result<()> {
        match self.def.kind() {
            IoTypeId::VisionSensor => Ok(()),
            _ => {
                Err(Error::HubError(String::from("Not a Vision sensor Motor")))
            }
        }
    }
}
