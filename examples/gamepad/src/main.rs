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
            println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);
            // println!("{}", &hub1_name);
            let n = NotificationMessage::parse(&data.value).unwrap();
            dbg!(&n);
        }  
    });



    let mut remote_a = hub1.port(lego_powered_up::hubs::Port::A).await?;
    // remote_b.request_port_info(lego_powered_up::notifications::InformationType::PortValue).await?;
    // remote_b.request_port_info(lego_powered_up::notifications::InformationType::ModeInfo).await?;
    // remote_b.request_port_info(lego_powered_up::notifications::InformationType::PossibleModeCombinations).await?;

    // remote_b.request_mode_info(0, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(1, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(2, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(3, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;
    // remote_b.request_mode_info(4, lego_powered_up::notifications::ModeInformationType::ValueFormat).await?;

    println!("remote buttons mode 0");
    remote_a.remote_buttons_enable(4, 1).await?;
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    // println!("remote buttons mode 1");
    // remote_b.remote_buttons_enable(1, 1).await?;
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



    // loop {}

    println!("Disconnect from hub `{}`", hub1.name().await?);
    hub1.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

