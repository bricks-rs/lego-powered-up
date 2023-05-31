use async_trait::async_trait;
use core::fmt::Debug;
use crate::{Error, Result};
use crate::notifications::{NotificationMessage, ValueFormatType, PortValueSingleFormat};
use crate::notifications::InputSetupSingle;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::IoTypeId;

// #[async_trait]
// pub trait SingleValueSensor: Debug + Send + Sync {
//     fn p(&self) -> Option<Peripheral>;
//     fn c(&self) -> Option<Characteristic>;
//     fn port(&self) -> u8;

//     async fn single_value_sensor_enable(&self, mode: u8, delta: u32) -> Result<()> {
//         let mode_set_msg =
//             NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
//                 port_id: self.port(),
//                 mode: mode as u8,
//                 delta,
//                 notification_enabled: true,
//             });
//         let p = match self.p() {
//             Some(p) => p,
//             None => return Err(Error::NoneError((String::from("Not a single value sensor"))))
//         };
//         crate::hubs::send(p, self.c().unwrap(), mode_set_msg).await
//     }
// }

#[async_trait]
pub trait Sensor8bit: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;
    fn check(&self, mode: u8) -> Result<()>;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;

    async fn enable_8bit_sensor(&self, mode: u8, delta: u32) -> Result<(broadcast::Receiver<Vec<i8>>, JoinHandle<()> )> {
        match self.check(mode) {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not an 8-bit sensor mode"))))
        }
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });

        let setmode = crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await;
            match setmode {
                Ok(()) => (),
                Err(e) => { return Err(Error::HubError((String::from("Error setting mode on device")))); }
            }

        // Set up channel
        let port_id = self.port();
        let (tx, mut rx) = broadcast::channel::<Vec<i8>>(8);
        match self.get_rx() {
            Ok(mut rx_from_main) => { 
                let task = tokio::spawn(async move {
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        tx.send(data.data);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError((String::from("Something went wrong"))))
        }

    }
}

#[async_trait]
pub trait Sensor16bit: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;
    fn check(&self, mode: u8) -> Result<()>;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;

    async fn enable_16bit_sensor(&self, mode: u8, delta: u32) -> Result<(broadcast::Receiver<Vec<i16>>, JoinHandle<()> )> {
        match self.check(mode) {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not a 16-bit sensor mode"))))
        }
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });

        let setmode = crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await;
            match setmode {
                Ok(()) => (),
                Err(e) => { return Err(Error::HubError((String::from("Error setting mode on device")))); }
            }

        // Set up channel
        let port_id = self.port();
        let (tx, mut rx) = broadcast::channel::<Vec<i16>>(8);
        match self.get_rx() {
            Ok(mut rx_from_main) => { 
                let task = tokio::spawn(async move {
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        let mut converted_data: Vec<i16> = Vec::new();

                        let chunks = data.data.chunks_exact(2);
                        for chunk in chunks {
                            converted_data.push( ((chunk[0] as i16) << 8) | chunk[1] as i16 );
                            // let number = ((chunk[0] as i16) << 8) | chunk[1] as i16;
                            // i16::from_le_bytes(chunk.into());
                        }
                        
                        tx.send(converted_data);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError((String::from("Something went wrong"))))
        }

    }
}


#[async_trait]
pub trait VisionSensor: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;

    async fn vison_sensor_single_enable(&self, mode: u8, delta: u32) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        let p = match self.p() {
            Some(p) => p,
            None => return Err(Error::NoneError((String::from("Not a Vision sensor"))))
        };
        crate::hubs::send(p, self.c().unwrap(), mode_set_msg).await
    }
}
