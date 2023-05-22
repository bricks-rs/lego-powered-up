// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use core::panic;
use std::println;

// use lego_powered_up::{notifications::Power, PoweredUp, btleplug::{platform::Manager}, devices};
use lego_powered_up::{PoweredUp, HubFilter, DiscoveredHub};
use lego_powered_up::{btleplug::api::Central};
// use btleplug::api::*;
// use std::time::Duration;

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
    let event_stream = adapter.events().await?;
    // Init PoweredUp with found adapter
    let mut pu = PoweredUp::with_adapter(adapter).await?;
    
    // let s = pu.scan().await?;

    



    let mut hubs = pu.wait_for_hubs_filter(HubFilter::Null, 2).await?.into_iter();
    let hub1 = hubs.next().unwrap();
    let hub2 = hubs.next().unwrap();

    // let hubs = pu.list_discovered_hubs().await?;

    // for h in hubs.into_iter() {
    //     println!("Hubtype: {}  Addr: {}  Name: {}", h.hub_type, h.addr, h.name);
    // }


    // println!("Listening for hubs...");
    // let mut pu = PoweredUp::init().await?;
    // let hub = pu.wait_for_hub().await?;

    // println!("Connecting to hub `{}`", hub.name);
    // let hub = pu.create_hub(&hub).await?;

    // println!("Change the hub LED to green");
    // let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed).await?;
    // hub_led.set_rgb(&[0, 0xff, 0]).await?;

    // println!("Run motors");
    // let mut motor_b = hub.port(lego_powered_up::hubs::Port::B).await?;
    // motor_b.start_speed(50, Power::Cw(50)).await?;

    // tokio::time::sleep(Duration::from_secs(3)).await;

    // println!("Stop motors");
    // motor_b.start_speed(0, Power::Float).await?;

    // println!("Disconnect from hub `{}`", hub.name().await?);
    // hub.disconnect().await?;

    println!("Done!");

    Ok(())
}
