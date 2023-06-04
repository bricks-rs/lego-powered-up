// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::time::Duration;
use tokio::time::sleep as sleep;

use lego_powered_up::setup;
use lego_powered_up::{PoweredUp, ConnectedHub, IoDevice, IoTypeId}; 
use lego_powered_up::{Hub, HubFilter, }; 
use lego_powered_up::error::{Error, Result, OptionContext}; 
use lego_powered_up::consts::{named_port, Color};
use lego_powered_up::notifications::Power;
use lego_powered_up::consts::{LEGO_COLORS, };
use lego_powered_up::devices::modes;
use lego_powered_up::devices::remote::{RcDevice, RcButtonState};
use lego_powered_up::devices::{light::*, sensor::*, motor::*, visionsensor::*};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === Single hub === 
    let hub = setup::single_hub().await?;

    let vision: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        vision = lock.io_from_kind(IoTypeId::VisionSensor).await?;
    }    

    let hubled: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hubled = lock.io_from_kind(IoTypeId::HubLed).await?;
    }

    // let (mut vision_rx, _) = vision.visionsensor_color().await.unwrap();
    // hubled.set_hubled_mode(HubLedMode::Colour).await.expect("Error setting mode");
    // hubled.set_hubled_color(Color::Black).await?;
    // let visionsensor_task = tokio::spawn(async move {
    //     while let Ok(data) = vision_rx.recv().await {
    //         println!("Vision: {:?} ", data, );
    //         match data {
    //             DetectedColor::Green => { hubled.set_hubled_color(Color::Green).await; },
    //             DetectedColor::Red => { hubled.set_hubled_color(Color::Red).await; },
    //             DetectedColor::Yellow => { hubled.set_hubled_color(Color::Yellow).await; },
    //             DetectedColor::Blue => { hubled.set_hubled_color(Color::Blue).await; },
    //             DetectedColor::White => { hubled.set_hubled_color(Color::White).await; },
    //             DetectedColor::Black => { hubled.set_hubled_color(Color::Black).await; },
    //             _ => ()
    //         }
    //         println!("Vision: {:?} ", data, )
    //     }
    // });

    hubled.set_hubled_mode(HubLedMode::Rgb).await.expect("Error setting mode");
    hubled.set_hubled_rgb(&[0x00, 0x00, 0x00]).await?;
    let (mut vision_rx, _) = vision.enable_16bit_sensor(modes::VisionSensor::RGB_I, 1).await?;
    let visionsensor_task = tokio::spawn(async move {
        while let Ok(data) = vision_rx.recv().await {
            hubled.set_hubled_rgb(&[data[0] as u8, data[1] as u8, data[2] as u8  ]).await;
            println!("Vision: {:?} ", data, )
        }
    });

    // color_task
    // rgb_task
    // raw_task


    let (mut vision_rx, _) = vision.enable_8bit_sensor(modes::VisionSensor::COLOR, 1).await?;
    let (mut vision_rx, _) = vision.enable_8bit_sensor(modes::VisionSensor::PROX, 1).await?;
    let (mut vision_rx, _) = vision.enable_32bit_sensor(modes::VisionSensor::COUNT, 1).await?;
    let (mut vision_rx, _) = vision.enable_8bit_sensor(modes::VisionSensor::REFLT, 1).await?;
    let (mut vision_rx, _) = vision.enable_8bit_sensor(modes::VisionSensor::AMBI, 1).await?;
    let (mut vision_rx, _) = vision.enable_16bit_sensor(modes::VisionSensor::RGB_I, 1).await?;
    let (mut vision_rx, _) = vision.enable_8bit_sensor(modes::VisionSensor::SPEC_1, 1).await?; // u8
    let (mut vision_rx, _) = vision.enable_16bit_sensor(modes::VisionSensor::DEBUG, 1).await?;
    let (mut vision_rx, _) = vision.enable_16bit_sensor(modes::VisionSensor::CALIB, 1).await?;



    visionsensor_task.await;
    

    
    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    Ok(())
}

