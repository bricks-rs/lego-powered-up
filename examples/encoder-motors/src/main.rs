// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/
#![allow(unused)]
use core::panic;
use std::println;

use lego_powered_up::{notifications::Power, PoweredUp, btleplug::{platform::Manager}, devices};
use lego_powered_up::{HubFilter, notifications::EndState, notifications::NotificationMessage};
// use lego_powered_up::{DiscoveredHub};
use lego_powered_up::{btleplug::api::{Central, CentralEvent, ScanFilter, Manager as _, Peripheral as _, PeripheralProperties}};
use lego_powered_up::{btleplug::api::Peripheral};
use lego_powered_up::{btleplug::api::{CharPropFlags, ValueNotification}};
use lego_powered_up::{futures::stream::{StreamExt, FuturesUnordered}};
use lego_powered_up::{futures::{future, select}};

// use btleplug::api::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // List adapters
    let adapters = PoweredUp::adapters().await; 
    match adapters {
        Result::Ok(list) => {
            if list.len() > 1 {
                println!("Found multiple adapters:\n");
                for a in list.into_iter() {
                    println!("{}",a.adapter_info().await?);
                }
                panic!("Multiple adapters found, don't know which to use.");
            } else {
                let a = list.into_iter().next().unwrap();
                println!("Found adapter: {}.", a.adapter_info().await?);
            }
        },    
        Err(error) => {
                println!("No adapters found: {}", error);
                panic!("No adapters found: {}", error);
            }
    }
    let adapter = PoweredUp::adapters().await?.into_iter().next().unwrap();
    // Init PoweredUp with found adapter
    let mut pu = PoweredUp::with_adapter(adapter).await?;

    let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, 1).await?;
   
    let mut dh_iter = discovered_hubs.into_iter();
    let dh1 = dh_iter.next().unwrap();
    println!("Connecting to hub `{}`", dh1.name);
    let hub1 = pu.create_hub(&dh1).await?;
    // dbg!(hub1.properties());
    let hub1_name = hub1.name().await?;
    let mut hub1_stream =
                        hub1.peripheral().notifications().await?;  


    tokio::spawn(async move {
        while let Some(data) = hub1_stream.next().await {
            // println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);
            let n = NotificationMessage::parse(&data.value).unwrap();
            println!("{}", &hub1_name);
            dbg!(&n);
        }  
    });



    let mut motor_b = hub1.port(lego_powered_up::hubs::Port::B).await?;

    motor_b.motor_sensor_enable(devices::MotorSensorMode::Speed, 10).await?;
    motor_b.start_speed(50, Power::Cw(50)).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    motor_b.start_speed(-35, Power::Cw(50)).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    motor_b.start_speed(70, Power::Cw(50)).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    motor_b.motor_sensor_enable(devices::MotorSensorMode::Angle, 180).await?;

    println!("Degrees 180 40 50 Brake");
    motor_b.start_speed_for_degrees(180, 40, Power::Cw(50), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Degrees 180 40 -50 Brake");
    motor_b.start_speed_for_degrees(180, -40, Power::Cw(50), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Position 360 40 50 Brake");
    motor_b.goto_absolute_position(360, 40, Power::Cw((50)), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Position 0 40 50 Brake");

    motor_b.motor_sensor_disable().await?;

    motor_b.goto_absolute_position(180, 40, Power::Cw((50)), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    motor_b.goto_absolute_position(360, 40, Power::Cw((50)), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    

   
    println!("Stop motors");
    motor_b.start_speed(0, Power::Float).await?;


    
   

    // loop {}
    // tokio::time::sleep(Duration::from_secs(30)).await;

    println!("Disconnect from hub `{}`", hub1.name().await?);
    hub1.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

