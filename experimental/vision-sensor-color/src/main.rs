// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]

// use core::time::Duration;
// use tokio::time::sleep as sleep;

use std::println;

use lego_powered_up::{PoweredUp, HubFilter, ConnectedHub, IoDevice, IoTypeId}; 
use lego_powered_up::consts::MotorSensorMode;
use lego_powered_up::consts::named_port;
use lego_powered_up::iodevice::motor::EncoderMotor;
use lego_powered_up::iodevice::remote::{RcDevice, RcButtonState};
use lego_powered_up::iodevice::sensor::GenericSensor;
use lego_powered_up::iodevice::visionsensor::*;
use lego_powered_up::iodevice::modes;
use lego_powered_up::notifications::Power;
use lego_powered_up::consts::Color;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let main_hub = lego_powered_up::setup::single_hub().await?;
    
    let vision: IoDevice;
    {
        let lock = main_hub.mutex.lock().await;
        vision = lock.io_from_kind(IoTypeId::VisionSensor).await?;
    }    

    vision.visionsensor_light_output_mode().await.expect("Error setting mode");

    vision.visionsensor_set_color(OutputColor::Blue).await?;
    tokio::time::sleep(core::time::Duration::from_millis(2000)).await;

    vision.visionsensor_set_color(OutputColor::Green).await?;
    tokio::time::sleep(core::time::Duration::from_millis(2000)).await;

    vision.visionsensor_set_color(OutputColor::Red).await?;
    tokio::time::sleep(core::time::Duration::from_millis(2000)).await;

    vision.visionsensor_set_color(OutputColor::White).await?;
    tokio::time::sleep(core::time::Duration::from_millis(2000)).await;

    vision.visionsensor_set_color(OutputColor::Off).await?;
    tokio::time::sleep(core::time::Duration::from_millis(2000)).await;


    println!("Disconnect from hub `{}`", main_hub.name);
    {
        let lock = main_hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    
    println!("Done!");

    Ok(())
}
