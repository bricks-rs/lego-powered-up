
use tokio::sync::broadcast;

use crate::hubs::{Channels, Tokens};
use crate::notifications::{
    DatasetType, NetworkCommand, PortValueCombinedFormat, PortValueSingleFormat, PortOutputCommandFeedbackFormat,
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

#[derive(Debug, Clone)]
pub struct IoDevice {
    pub def: Definition,
    tokens: Tokens,
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
    pub fn channels(&self) -> &Channels {
        &self.channels
    }
    pub fn new(kind: IoTypeId, port: u8, tokens: Tokens) -> Self {
        Self {
            def: Definition::new(kind, port),
            tokens,
            channels: Default::default(),
        }
    }
    // pub fn new(kind: IoTypeId, port: u8) -> Self {
    //     Self {
    //         def: Definition::new(kind, port),
    //         tokens: Default::default(),
    //         channels: Default::default(),
    //     }
    // }
    // pub fn cache_tokens(
    //     &mut self,
    //     tokens: (Option<Arc<Peripheral>>, Option<Arc<Characteristic>>),
    // ) {
    //     (self.tokens.p, self.tokens.c) = tokens;
    // }
    pub fn cache_channels(&mut self, channels: Channels) {
        self.channels = channels;
    }

}

//
// Implement device-traits
//
impl Basic for IoDevice {
    fn port(&self) -> u8 {
        self.def.port()
    }
    fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }
    // fn check(&self) -> Result<()> {
    //    Ok(())
    // }
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
        if let Some(sender) = &self.channels.singlevalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found")))
        }
    }

}
impl GenericSensor for IoDevice {
    // fn port(&self) -> u8 {
    //     self.def.port()
    // }
    // fn tokens(&self) -> Tokens {
    //     self.tokens.clone()
    // }
    // fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
    //     if let Some(sender) = &self.channels.singlevalue_sender {
    //         Ok(sender.subscribe())
    //     } else {
    //         Err(Error::NoneError(String::from("Sender not found")))
    //     }
    // }
    fn check(&self) -> Result<()> {
        Ok(())
     }
    fn check_dataset(&self, mode: u8, datasettype: DatasetType) -> Result<()> {
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
    // fn port(&self) -> u8 {
    //     self.def.port()
    // }
    // fn tokens(&self) -> Tokens {
    //     self.tokens.clone()
    // }
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
    // fn port(&self) -> u8 {
    //     self.def.port()
    // }
    // fn tokens(&self) -> Tokens {
    //     self.tokens.clone()
    // }
    // fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
    //     if let Some(sender) = &self.channels.singlevalue_sender {
    //         Ok(sender.subscribe())
    //     } else {
    //         Err(Error::NoneError(String::from("Sender not found")))
    //     }
    // }
    fn get_rx_combined(
        &self,
    ) -> Result<broadcast::Receiver<PortValueCombinedFormat>> {
        if let Some(sender) = &self.channels.combinedvalue_sender {
            Ok(sender.subscribe())
        } else {
            Err(Error::NoneError(String::from("Sender not found")))
        }
    }
    fn get_rx_feedback(&self) -> Result<broadcast::Receiver<PortOutputCommandFeedbackFormat>> {
        if let Some(sender) = &self.channels.commandfeedback_sender {
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
    // fn port(&self) -> u8 {
    //     self.def.port()
    // }
    // fn tokens(&self) -> Tokens {
    //     self.tokens.clone()
    // }
    fn check(&self) -> Result<()> {
        match self.def.kind() {
            IoTypeId::HubLed => Ok(()),
            _ => Err(Error::HubError(String::from("Not a Hub LED device"))),
        }
    }
}

impl VisionSensor for IoDevice {
    // fn port(&self) -> u8 {
    //     self.def.port()
    // }
    // fn tokens(&self) -> Tokens {
    //     self.tokens.clone()
    // }
    // fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>> {
    //     if let Some(sender) = &self.channels.singlevalue_sender {
    //         Ok(sender.subscribe())
    //     } else {
    //         Err(Error::NoneError(String::from("Sender not found")))
    //     }
    // }
    fn check(&self) -> Result<()> {
        match self.def.kind() {
            IoTypeId::VisionSensor => Ok(()),
            _ => {
                Err(Error::HubError(String::from("Not a Vision sensor Motor")))
            }
        }
    }
}

#[macro_export]
macro_rules! device_trait {
    ($name:tt, [$( $b:item ),*])  => { 
        #[async_trait]
        pub trait $name: Debug + Send + Sync + Basic {
            // fn port(&self) -> u8;
            fn check(&self) -> Result<()>;
            // fn tokens(&self) -> Tokens;
            // async fn commit(&self, msg: NotificationMessage) -> Result<()> {
            //     match crate::hubs::send(self.tokens(), msg).await {
            //         Ok(()) => Ok(()),
            //         Err(e) => Err(e),
            //     }
            // }
            $(
                $b
            )*
        }
    }
}
