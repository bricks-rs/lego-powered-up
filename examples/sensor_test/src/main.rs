// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;
use lego_powered_up::devices::iodevice::IoDevice;
use lego_powered_up::devices::visionsensor::VisionSensor;
use lego_powered_up::notifications::Power;
use tokio::time::sleep as sleep;

// Powered up
use lego_powered_up::{PoweredUp, Hub, HubFilter, ConnectedHub,}; 
use lego_powered_up::consts::{IoTypeId, LEGO_COLORS};
use lego_powered_up::devices::remote::RcDevice;
use lego_powered_up::devices::{light::*, sensor::*, motor::*};


// Access hub 
use std::sync::{Arc};
use tokio::sync::Mutex;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;

// / Access devices
use lego_powered_up::{ error::Error};  //devices::Device,
use tokio::sync::broadcast;
use lego_powered_up::devices::modes;

// RC
// use lego_powered_up::hubs::remote::*;

// Handle notifications
use core::pin::Pin;
use lego_powered_up::futures::stream::{Stream, StreamExt};
use lego_powered_up::btleplug::api::ValueNotification;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rc_hub = lego_powered_up::setup::single_hub().await?;

    let main_hub: ConnectedHub;

    let mut ferry: Vec<IoDevice> = Vec::new();

    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     ferry = lock.io_multi_from_kind(IoTypeId::Rssi).await?;
    // } 
    // let rssi = ferry.remove(0);   
    // let (mut rssi_rx, jh) = rssi.enable_8bit_sensor(0x00, 1).await.unwrap();
    // tokio::spawn(async move {
    //     while let Ok(data) = rssi_rx.recv().await {
    //         println!("Rssi: {:?} {:?}", data, data[0] as i8)
    //     }
    // });

    // let mut remote_a: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     remote_a = lock.get_from_port(0).await?;
    // }    
    // let (mut remote_a_rx, jh) = remote_a.enable_8bit_sensor(0x00, 1).await.unwrap();
    // tokio::spawn(async move {
    //     while let Ok(data) = remote_a_rx.recv().await {
    //         println!("Remote_a: {:?}", data)
    //     }
    // });

    let mut rc_volt: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc_volt = lock.io_from_kind(IoTypeId::Voltage).await?;
    }    
    let (mut voltage_rx, jh) = rc_volt.enable_16bit_sensor(0x00, 1).await.unwrap();
    tokio::spawn(async move {
        while let Ok(data) = voltage_rx.recv().await {
            // println!("Voltage: {:?}  {:?}", data, data[0] as u16)
        }
    });

    // let mut move_accel: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     move_accel = lock.get_from_kind(IoTypeId::InternalTilt).await?;
    // }    
    // let (mut move_accel_rx, jh) = move_accel.enable_8bit_sensor(0x04, 1).await.unwrap();
    // tokio::spawn(async move {
    //     while let Ok(data) = move_accel_rx.recv().await {
    //         println!("Accel: {:?}  {:?}", data, data[0] as u16)
    //     }
    // });

    // let mut move_impct: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     move_impct = lock.get_from_kind(IoTypeId::InternalTilt).await?;
    // }    
    // let (mut move_impct_rx, jh) = move_impct.enable_32bit_sensor(0x03, 1).await.unwrap();
    // tokio::spawn(async move {
    //     while let Ok(data) = move_impct_rx.recv().await {
    //         println!("Accel: {:?}  {:?}", data, data[0] as u16)
    //     }
    // });

    // let mut motor_b: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     motor_b = lock.io_from_port(1).await?;
    // }    
    // let (mut motor_b_rx, jh) = motor_b.enable_32bit_sensor(modes::Voltage::VLT_L, 1).await.unwrap();
    // // tokio::spawn(async move {
    // //     while let Ok(data) = motor_b_rx.recv().await {
    // //         println!("Combined: {:?}  ", data, );
    // //     }
    // // });
    // let (mut motor_b_rx, jh) = 
    //     motor_b.motor_combined_sensor_enable(lego_powered_up::consts::MotorSensorMode::Speed, 2, 2).await.unwrap();
    // tokio::spawn(async move {
    //     while let Ok(data) = motor_b_rx.recv().await {
    //         println!("Combined: {:?}  ", data, );
    //     }
    // });
    // motor_b.start_speed_for_degrees(90, 50, Power::Cw((50)), lego_powered_up::notifications::EndState::Brake).await;
    // sleep(Duration::from_secs(2)).await;
    // motor_b.start_speed_for_degrees(90, 50, Power::Cw((50)), lego_powered_up::notifications::EndState::Brake).await;
    // sleep(Duration::from_secs(2)).await;
    // motor_b.start_speed_for_degrees(890, -50, Power::Cw((50)), lego_powered_up::notifications::EndState::Brake).await;

    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     ferry = lock.io_multi_from_kind(IoTypeId::RemoteButtons).await?;
    // }   
    // let remote_a = ferry.remove(0);
    // let remote_b = ferry.remove(0);
    // println!("RC A: {:?} {:?}", remote_a.kind, remote_a.port);
    // println!("RC B: {:?} {:?}", remote_b.kind, remote_b.port);

    // let vision: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     vision = lock.io_from_kind(IoTypeId::VisionSensor).await?;
    // }    
    // // let (mut vision_rx, _) = vision.enable_8bit_sensor(modes::VisionSensor::COLOR, 1).await.unwrap();
    // let (mut vision_rx, _) = vision.visionsensor_color().await.unwrap();
    // tokio::spawn(async move {
    //     while let Ok(data) = vision_rx.recv().await {
    //         println!("Vision: {:?} ", data, )
    //     }
    // });

    let mut vision: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        vision = lock.io_from_kind(IoTypeId::VisionSensor).await?;
    }   
    let sensormode = modes::VisionSensor::RGB_I ;
    let (mut vision_rx, jh) = vision.enable_16bit_sensor(sensormode, 1).await.unwrap();
    let sensor_task = tokio::spawn(async move {
        while let Ok(data) = vision_rx.recv().await {
            println!("Vision {:?}  {:?}", sensormode, data);
        }
    });
    sensor_task.await?;


    // Start ui
    let mutex = rc_hub.mutex.clone();
    ui(mutex).await;

    // Cleanup after ui exit
    let lock = rc_hub.mutex.lock().await;
    println!("Disconnect from hub `{}`", lock.name().await?);
    lock.disconnect().await?;
    println!("Done!");

    Ok(())
}


pub async fn ui(mutex: HubMutex) -> () {
    use text_io::read;
    loop {
        print!("(l)ist, <port> or (q)uit > ");
        let line: String = read!("{}\n");
        if line.len() == 1 {
            continue;
        } 
        else if line.contains("l") {
            let mut lock = mutex.lock().await;
            for device in lock.connected_io().values() {
                println!("{}", device);
            }
            continue;
        } 
        else if line.contains("q") {
            break
        }
        else {
            let input = line.trim().parse::<u8>();
            // println!("Input: {}", input);
            match input {
                Ok(num) => {
                    let mut lock = mutex.lock().await;
                    let o = lock.connected_io().get(&num);
                    match o {
                        Some(device) => {dbg!(device);}
                        None => {println!("Device not found");}
                    }
                }
                Err(e) => {
                    println!("Not a number: {}", e);
                }
            }
        }
    }
}
