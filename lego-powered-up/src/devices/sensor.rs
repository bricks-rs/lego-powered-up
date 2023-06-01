
use crate::error::{Error, OptionContext, Result};
use async_trait::async_trait;
use core::fmt::Debug;
use std::convert;
use crate::notifications::{NotificationMessage, ValueFormatType, PortValueSingleFormat, DatasetType};
use crate::notifications::InputSetupSingle;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::IoTypeId;

#[macro_use]
use crate::notifications::macros::*;


#[async_trait]
pub trait GenericSensor: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;
    // fn check(&self, mode: u8) -> Result<()>;
    fn check(&self, mode: u8, datasettype: DatasetType) -> Result<()>;
    fn get_rx(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;

    async fn set_device_mode(&self, mode: u8, delta: u32) -> Result<()>{
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });

        let setmode = crate::hubs::send(self.p().unwrap(), self.c().unwrap(), mode_set_msg).await;
            match setmode {
                Ok(()) => Ok(()),
                Err(e) => { return Err(Error::HubError((String::from("Error setting mode on device")))); }
            }
    }

    async fn enable_8bit_sensor(&self, mode: u8, delta: u32) -> Result<(broadcast::Receiver<Vec<i8>>, JoinHandle<()> )> {
        match self.check(mode, DatasetType::Bits8) {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not an 8-bit sensor mode"))))
        }
        self.set_device_mode(mode, delta).await?;

        // Set up channel
        let port_id = self.port();
        let (tx, mut rx) = broadcast::channel::<Vec<i8>>(8);
        match self.get_rx() {
            Ok(mut rx_from_main) => { 
                let task = tokio::spawn(async move {
                    while let Ok(msg) = rx_from_main.recv().await {
                        if msg.port_id != port_id {
                            continue;
                        }
                        tx.send(msg.data);
                        
                        // let converted_data = data.data.into_iter().map(|x| x as i8).collect();
                        // tx.send(converted_data);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError((String::from("No sender in device cache"))))
        }

    }

    async fn enable_16bit_sensor(&self, mode: u8, delta: u32) -> Result<(broadcast::Receiver<Vec<i16>>, JoinHandle<()> )> {
        match self.check(mode, DatasetType::Bits16) {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not a 16-bit sensor mode"))))
        }
        self.set_device_mode(mode, delta).await?;

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
                        let mut converted: Vec<i16> = Vec::new();

                        // let it = data.data.into_iter().map(|x| x as u8);
                        // converted.push(next_i16!(it));                       

                        // let chunks = data.data.chunks_exact(2);
                        // for c in chunks {
                        //     converted.push( ((c[0] as u16) << 8) | (c[1] as u16) );
                        // }
                        // let converted_2 = converted.into_iter().map(|x| x as i16).collect();

                        let cycles = &data.data.len() / 2;
                        let mut it = data.data.into_iter();
                        for _ in 0..cycles {
                            converted.push(i16::from_le_bytes([it.next().unwrap() as u8, it.next().unwrap() as u8]));
                        }    

                        tx.send(converted);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError((String::from("No sender in device cache"))))
        }
    }

    async fn enable_32bit_sensor(&self, mode: u8, delta: u32) -> Result<(broadcast::Receiver<Vec<i32>>, JoinHandle<()> )> {
        match self.check(mode, DatasetType::Bits32) {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not a 32-bit sensor mode"))))
        }
        self.set_device_mode(mode, delta).await?;

        // Set up channel
        let port_id = self.port();
        let (tx, mut rx) = broadcast::channel::<Vec<i32>>(8);
        match self.get_rx() {
            Ok(mut rx_from_main) => { 
                let task = tokio::spawn(async move {
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        // println!("32bit data: {:?}", &data.data);

                        let mut converted: Vec<i32> = Vec::new();

                        let cycles = &data.data.len() / 4;
                        let mut it = data.data.into_iter();
                        for _ in 0..cycles {
                            converted.push(i32::from_le_bytes([it.next().unwrap() as u8, it.next().unwrap() as u8,
                                                               it.next().unwrap() as u8, it.next().unwrap() as u8]));
                        }    

                        tx.send(converted);
                    }
                });

                Ok((rx, task))
            }
            _ => Err(Error::NoneError((String::from("No sender in device cache"))))
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
