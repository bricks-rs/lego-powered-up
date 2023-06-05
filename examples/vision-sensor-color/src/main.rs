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
    // let (main_hub, rc_hub) = lego_powered_up::setup::main_and_rc().await?;
    
    // Set up RC input 
    // let rc: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     rc = lock.io_from_port(named_port::A).await?;
    // }    
    // let (mut rc_rx, rc_task) = rc.remote_connect_with_green().await?;
    
    let vision: IoDevice;
    {
        let lock = main_hub.mutex.lock().await;
        vision = lock.io_from_kind(IoTypeId::VisionSensor).await?;
    }    
    let (mut vision_rx, _) = vision.visionsensor_color().await.unwrap();
    vision.visionsensor_light_mode().await.expect("Error setting mode");
    // vision.visionsensor_set_color(Color::Black).await?;

    for c in 0..20 {
        println!("Set color: {:?}", c);
        vision.visionsensor_set_color(c as i8).await?;
        tokio::time::sleep(core::time::Duration::from_millis(1000)).await;
    }


    // Control task
    // let light_control = tokio::spawn(async move {
    //     loop {
    //         tokio::select! {
    //             Ok(msg) = rc_rx.recv() => {
    //                 match msg {
    //                     RcButtonState::Aup => { 
    //                         vision.visionsensor_set_color(Color::Black).await;
    //                     }
    //                     RcButtonState::Aminus => { 
    //                         vision.visionsensor_set_color(Color::Blue).await;
    //                     }
    //                     RcButtonState::Aplus => { 
    //                         vision.visionsensor_set_color(Color::Yellow).await;
    //                     }
    //                     RcButtonState::Ared => { 
    //                         vision.visionsensor_set_color(Color::Red).await;
    //                     }
    //                     RcButtonState::Bred => { 
    //                     }
    //                     RcButtonState::Bplus => { 
    //                         vision.visionsensor_set_color(Color::Green).await;
    //                     }
    //                     RcButtonState::Bminus => { 
    //                         vision.visionsensor_set_color(Color::White).await;
    //                     }

    //                     // RcButtonState::Bup => { println!("B side released"); }
    //                     RcButtonState::Green => { 
    //                         println!("Exiting");
    //                         break;
    //                     }
    //                     // RcButtonState::GreenUp => { println!("Green released") }
    //                     _ => ()
    //                 }
    //             }
     
    //             else => { break }    
    //         };
    //     }    
    // });
    // light_control.await;

    // Cleanup 
    // println!("Disconnect from hub `{}`", rc_hub.name);
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     lock.disconnect().await?;
    // }
    println!("Disconnect from hub `{}`", main_hub.name);
    {
        let lock = main_hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    
    println!("Done!");

    Ok(())
}
