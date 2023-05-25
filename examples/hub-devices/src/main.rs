// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/
#![allow(unused)]
use core::panic;
use std::println;

use lego_powered_up::{notifications::{Power, PortInformationType, InformationType, ModeInformationType}, PoweredUp, btleplug::{platform::Manager}, devices::{self, MotorSensorMode}};
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

    let hub_count = 1;
    println!("Waiting for hubs...");
    let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, &hub_count).await?;
    println!("Discovered {} hubs, trying to connect...", &hub_count);

   
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
            println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);

            let r = NotificationMessage::parse(&data.value);
            match r {
                Ok(n) => {
                    println!("{}", &hub1_name);
                    dbg!(&n);
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                }
            }

        }  
    });

    // Move hub internal devices, port adress 
    //      Internal motor A, 0x0
    //      Internal motor B, 0x1
    //      Virtual combined motor AB, 0x10
    //      HubLed, 0x32
    //      TiltSensor, 0x3a
    //      CurrentSensor, 0x3b
    //      VoltageSensor, 0x3c
    //      Mystery device, 0x46                  // Unknown device on this port. Its 3 modes are named TRIGGER, CANVAS and VAR

    // if &hub1_name.eq("LEGO Move Hub") {
    // let ports: Vec<u8> = vec![0x32, 0x3a, 0x3b, 0x3c,];
    // for p in ports.into_iter() {
    //     hub1.request_port_info(p, InformationType::PortValue).await?;
    //     hub1.request_port_info(p, InformationType::ModeInfo).await?;
    //     hub1.request_port_info(p, InformationType::PossibleModeCombinations).await?;
    // }

    // let ports: Vec<u8> = vec![0x32, 0x3b, 0x3c,];
    // for p in ports.into_iter() {
    //     let modes: Vec<u8> = vec![0, 1];
    //     for m in modes.into_iter() {
    //         hub1.request_mode_info(p, m, ModeInformationType::Name).await?;   
    //     }
    // }

    // let modes: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    //     for m in modes.into_iter() {
    //         hub1.request_mode_info(0x3a, m, ModeInformationType::Name).await?;   
    //     }
    
   hub1.set_port_mode(0x3a, 2, 1, true).await?;


    // Remote handset devices, port adress
    //      Button cluster A, 0x0
    //      Button cluster B, 0x1
    //      HubLed, 0x34
    //      VoltageSensor, 0x3b
    //      Rssi, 0x3c
   
    // Technic Medium hub internal devices, port adress
    //      HubLed, 0x32
    //      CurrentSensor, 0x3b
    //      VoltageSensor, 0x3c
    //      Accelerometer, 0x61
    //      GyroSensor, 0x62
    //      TiltSensor, 0x63
    //      GestureSensor, 0x64
    //      TemperatureSensor1, 0x60
    //      TemperatureSensor2, 0x3d


    // loop {}
    tokio::time::sleep(Duration::from_secs(30)).await;

    println!("Disconnect from hub `{}`", hub1.name().await?);
    hub1.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

