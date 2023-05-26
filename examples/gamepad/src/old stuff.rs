// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/
#![allow(unused)]
use core::panic;
use std::println;
use std::time::Duration;


use lego_powered_up::{PoweredUp, HubFilter, devices,};
use lego_powered_up::notifications::{NotificationMessage, Power, EndState, 
                                    InformationType, ModeInformationType,
                                    HubAction, PortValueSingleFormat};
use lego_powered_up::consts::*;
// use lego_powered_up::{DiscoveredHub};

// Btleplug reexports, provide abstractions for these
use lego_powered_up::{btleplug::api::{Central, CentralEvent, ScanFilter, Manager as _, Peripheral as _, PeripheralProperties}};
use lego_powered_up::{btleplug::api::Peripheral};
use lego_powered_up::{btleplug::api::{CharPropFlags, ValueNotification}};
use lego_powered_up::{btleplug::platform::{Manager}};

use lego_powered_up::{futures::stream::{StreamExt, FuturesUnordered}};
use lego_powered_up::{futures::{future, select}};


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

    let mut remote_status = lego_powered_up::hubs::remote::RemoteStatus::new();


    tokio::spawn(async move {
        while let Some(data) = hub1_stream.next().await {
            println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);

            let r = NotificationMessage::parse(&data.value);
            match r {
                Ok(n) => {
                    // println!("{}", &hub1_name);
                    // dbg!(&n);
                    match n {
                        NotificationMessage::PortValueSingle(val) => {
                            match val.values[0] {
                                0x0 => {
                                    match val.values[1] {
                                        0 => {
                                            remote_status.a_plus = false;
                                            remote_status.a_red = false; 
                                            remote_status.a_minus = false;
                                            println!("A-button released");
                                        }
                                        1 => {
                                            remote_status.a_plus = true;
                                            remote_status.a_red = false; 
                                            remote_status.a_minus = false;
                                            println!("A+ pressed");
                                        }
                                        127 => {
                                            remote_status.a_plus = false;
                                            remote_status.a_red = true; 
                                            remote_status.a_minus = false;
                                            println!("Ared pressed");
                                        }
                                        255 => {
                                            remote_status.a_plus = false;
                                            remote_status.a_red = false; 
                                            remote_status.a_minus = true;
                                            println!("A- pressed");
                                        }
                                        _  => ()
                                    }
                                    // dbg!(remote_status);
                                }
                                _ => ()                                
                            }
                            
                        }
                        _ => ()
                    }
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                }
            }
        }  
    });



    // let req = hub1.request_port_info(0x46, InformationType::ModeInfo).await;
    // match req {
    //     Ok(()) => (),
    //     Err(error) => println!("Request error: {}", &error)
    // }

    // hub1.request_port_info(0x2, InformationType::ModeInfo).await?;
    // hub1.request_port_info(0x2, InformationType::PossibleModeCombinations).await?;

    // hub1.request_mode_info(0x2, 1, ModeInformationType::ValueFormat).await?;
    // hub1.request_mode_info(0x2, 2, ModeInformationType::ValueFormat).await?;
    // hub1.request_mode_info(0x2, 3, ModeInformationType::ValueFormat).await?;
    // hub1.request_mode_info(0x3, 8, ModeInformationType::Name).await?;
    // hub1.request_mode_info(0x3, 9, ModeInformationType::Name).await?;
    // hub1.request_mode_info(0x3, 10, ModeInformationType::Name).await?;

    // let mut remote_a = hub1.port(lego_powered_up::hubs::Port::A).await?;
    // remote_b.request_port_info(lego_powered_up::notifications::InformationType::PortValue).await?;
    // remote_b.request_port_info(lego_powered_up::notifications::InformationType::ModeInfo).await?;
    // remote_b.request_port_info(lego_powered_up::notifications::InformationType::PossibleModeCombinations).await?;

    // remote_b.request_mode_info(0, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(1, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(2, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(3, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(4, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;

    // println!("remote buttons mode 0");
    // remote_a.remote_buttons_enable(0, 1).await?;
    // tokio::time::sleep(Duration::from_secs(10)).await;
    
    println!("remote buttons A mode 1");
    let mut remote_a = hub1.port(lego_powered_up::hubs::Port::A).await?;
    remote_a.remote_buttons_enable(1, 1).await?;
    // tokio::time::sleep(Duration::from_secs(10)).await;

    println!("remote buttons B mode 1");
    let mut remote_b = hub1.port(lego_powered_up::hubs::Port::B).await?;
    remote_b.remote_buttons_enable(1, 1).await?;
    // tokio::time::sleep(Duration::from_secs(10)).await;

    // println!("remote buttons mode 2");
    // remote_b.remote_buttons_enable(2, 1).await?;
    // tokio::time::sleep(Duration::from_secs(10)).await;

    // println!("remote buttons mode 3");
    // remote_b.remote_buttons_enable(3, 1).await?;
    // tokio::time::sleep(Duration::from_secs(10)).await;

    // println!("remote buttons mode 4");
    // remote_b.remote_buttons_enable(4, 1).await?;
    // tokio::time::sleep(Duration::from_secs(10)).await;



    loop {}

    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Disconnect from hub `{}`", hub1.name().await?);
    hub1.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

