// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/
#![allow(unused)]
use core::panic;
use std::println;

use lego_powered_up::{notifications::Power, PoweredUp, btleplug::{platform::Manager}, devices};
use lego_powered_up::{HubFilter, notifications::EndState};
// use lego_powered_up::{DiscoveredHub};
use lego_powered_up::{btleplug::api::Central};
use lego_powered_up::{btleplug::api::Peripheral};

// use btleplug::api::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get btleplug manager
    // let manager = Manager::new().await?;
    
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
    // let adapter_event_stream = adapter.events().await?;
    // Init PoweredUp with found adapter
    let mut pu = PoweredUp::with_adapter(adapter).await?;
    
    // let s = pu.scan().await?;

    
    


    let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, 1).await?;
   
    // for h in discovered_hubs.into_iter() {

    // }

    // let hubs: [DiscoveredHub; 2];
    
    let mut dh_iter = discovered_hubs.into_iter();
    let dh1 = dh_iter.next().unwrap();
    println!("Connecting to hub `{}`", dh1.name);
    let hub1 = pu.create_hub(&dh1).await?;
    let hub1_props = hub1.properties().await;
    dbg!(&hub1_props);



    // let dh2 = dh_iter.next().unwrap();
    // println!("Connecting to hub `{}`", dh2.name);
    // let hub2 = pu.create_hub(&dh2).await?;


    // let hub1 = pu.create_hub(&discovered_hubs.next().unwrap()).await?;
    // let hub2 = pu.create_hub(&discovered_hubs.next().unwrap()).await?;



    // let hubs = pu.list_discovered_hubs().await?;

    // for h in hubs.into_iter() {
    //     println!("Hubtype: {}  Addr: {}  Name: {}", h.hub_type, h.addr, h.name);
    // }




    println!("Change the hub1 LED to green");
    let mut hub_led1 = hub1.port(lego_powered_up::hubs::Port::HubLed).await?;
    hub_led1.set_rgb(&[0, 0xff, 0]).await?;

    // println!("Change the hub2 LED to green");
    // let mut hub_led2 = hub2.port(lego_powered_up::hubs::Port::HubLed).await?;
    // hub_led2.set_rgb(&[0, 0xff, 0]).await?;

    // println!("Run motors");
    let mut motor_b = hub1.port(lego_powered_up::hubs::Port::B).await?;
   
    // motor_b.start_speed(50, Power::Cw(50)).await?;
    // tokio::time::sleep(Duration::from_secs(3)).await;
   
    motor_b.start_speed_for_degrees(90, 40, Power::Cw(50), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    motor_b.start_speed_for_degrees(90, 40, Power::Cw(50), EndState::Brake).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    // motor_b.goto_absolute_position(0, 40, Power::Cw((50)), EndState::Hold).await?;

    // println!("Stop motors");
    // motor_b.start_speed(0, Power::Float).await?;

    // println!("Disconnect from hub `{}`", hub.name().await?);
    


    hub1.disconnect().await?;
    // hub2.disconnect().await?;


    println!("Done!");

    Ok(())
}
