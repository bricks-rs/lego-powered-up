//! Support for generic sensors. Can be used with simple sensors
//! like hub temp, voltage etc., or other devices without
//! higher level support.

use async_trait::async_trait;
use core::fmt::Debug;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::Basic;
use crate::device_trait;
use crate::error::{Error, Result};
use crate::notifications::DatasetType;

device_trait!(GenericSensor, [
    fn check_dataset(&self, mode: u8, datasettype: DatasetType) -> Result<()>;,

    async fn enable_8bit_sensor(
        &self,
        mode: u8,
        delta: u32,
    ) -> Result<(broadcast::Receiver<Vec<i8>>, JoinHandle<()>)> {
        match self.check_dataset(mode, DatasetType::Bits8) {
            Ok(()) => (),
            _ => {
                return Err(Error::NoneError(String::from(
                    "Not an 8-bit sensor mode",
                )))
            }
        }

        self.device_mode(mode, delta, true).await?;

        // Set up channel
        let port_id = self.port();
        let (tx, rx) = broadcast::channel::<Vec<i8>>(64);
        match self.get_rx() {
            Ok(mut rx_from_main) => {
                let task = tokio::spawn(async move {
                    while let Ok(msg) = rx_from_main.recv().await {
                        if msg.port_id != port_id {
                            continue;
                        }
                        let _ = tx.send(msg.data);
                    }
                });

                Ok((rx, task))
            }
            _ => {
                Err(Error::NoneError(String::from("No sender in device cache")))
            }
        }
    },

    async fn enable_16bit_sensor(
        &self,
        mode: u8,
        delta: u32,
    ) -> Result<(broadcast::Receiver<Vec<i16>>, JoinHandle<()>)> {
        match self.check_dataset(mode, DatasetType::Bits16) {
            Ok(()) => (),
            _ => {
                return Err(Error::NoneError(String::from(
                    "Not a 16-bit sensor mode",
                )))
            }
        }
        self.device_mode(mode, delta, true).await?;

        // Set up channel
        let port_id = self.port();
        let (tx, rx) = broadcast::channel::<Vec<i16>>(64);
        match self.get_rx() {
            Ok(mut rx_from_main) => {
                let task = tokio::spawn(async move {
                    while let Ok(data) = rx_from_main.recv().await {
                        if data.port_id != port_id {
                            continue;
                        }
                        let mut converted: Vec<i16> = Vec::new();


                        let cycles = &data.data.len() / 2;
                        let mut it = data.data.into_iter();
                        for _ in 0..cycles {
                            converted.push(i16::from_le_bytes([
                                it.next().unwrap() as u8,
                                it.next().unwrap() as u8,
                            ]));
                        }

                        let _ = tx.send(converted);
                    }
                });

                Ok((rx, task))
            }
            _ => {
                Err(Error::NoneError(String::from("No sender in device cache")))
            }
        }
    },

    async fn enable_32bit_sensor(
        &self,
        mode: u8,
        delta: u32,
    ) -> Result<(broadcast::Receiver<Vec<i32>>, JoinHandle<()>)> {
        match self.check_dataset(mode, DatasetType::Bits32) {
            Ok(()) => (),
            _ => {
                return Err(Error::NoneError(String::from(
                    "Not a 32-bit sensor mode",
                )))
            }
        }
        self.device_mode(mode, delta, true).await?;

        // Set up channel
        let port_id = self.port();
        let (tx, rx) = broadcast::channel::<Vec<i32>>(64);
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
                            converted.push(i32::from_le_bytes([
                                it.next().unwrap() as u8,
                                it.next().unwrap() as u8,
                                it.next().unwrap() as u8,
                                it.next().unwrap() as u8,
                            ]));
                        }

                        let _ = tx.send(converted);
                    }
                });

                Ok((rx, task))
            }
            _ => {
                Err(Error::NoneError(String::from("No sender in device cache")))
            }
        }
    },

    fn raw_channel(
        &self,
    ) -> Result<(broadcast::Receiver<Vec<i8>>, JoinHandle<()>)> {
        let port_id = self.port();
        let (tx, rx) = broadcast::channel::<Vec<i8>>(64);
        match self.get_rx() {
            Ok(mut rx_from_main) => {
                let task = tokio::spawn(async move {
                    while let Ok(msg) = rx_from_main.recv().await {
                        if msg.port_id != port_id {
                            continue;
                        }
                        let _ = tx.send(msg.data);
                    }
                });

                Ok((rx, task))
            }
            _ => {
                Err(Error::NoneError(String::from("No sender in device cache")))
            }
        }
    }

]);
