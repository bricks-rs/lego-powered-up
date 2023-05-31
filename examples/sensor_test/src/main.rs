// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;
use lego_powered_up::devices::iodevice::IoDevice;
use tokio::time::sleep as tokiosleep;

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
use lego_powered_up::{devices::Device, error::Error};
use tokio::sync::broadcast;

// RC
use lego_powered_up::hubs::remote::*;

// Handle notifications
use core::pin::Pin;
use lego_powered_up::futures::stream::{Stream, StreamExt};
use lego_powered_up::btleplug::api::ValueNotification;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init PoweredUp with found adapter
    println!("Looking for BT adapter and initializing PoweredUp with found adapter");
    let mut pu = PoweredUp::init().await?;

    let hub_count = 1;
    println!("Waiting for hubs...");
    let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, &hub_count).await?;
    println!("Discovered {} hubs, trying to connect...", &hub_count);

    let mut h: Vec<ConnectedHub> = Vec::new();
    for dh in discovered_hubs {
        println!("Connecting to hub `{}`", dh.name);
        let created_hub = pu.create_hub(&dh).await?;
        h.push(ConnectedHub::setup_hub(created_hub).await)
    }
    tokiosleep(Duration::from_secs(2)).await;

    let rc_hub: ConnectedHub = h.remove(0);
    let main_hub: ConnectedHub;
    // match h[0].kind {
    //     lego_powered_up::consts::HubType::RemoteControl => {
    //         rc_hub = h.remove(0);
    //         if h.len() > 0 { main_hub = h.remove(0) }
    //     }
    //     _ => {
    //         main_hub = h.remove(0);
    //         if h.len() > 0 { rc_hub = h.remove(0) }
    //     }
    // }

    let mut rssi: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rssi = lock.get_from_kind(IoTypeId::Rssi).await?;
    }    
    let (mut rssi_rx, jh) = rssi.enable_8bit_sensor(0x00, 1).await.unwrap();

    let mut remote_a: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        remote_a = lock.get_from_port(0).await?;
    }    
    let (mut remote_a_rx, jh) = remote_a.enable_8bit_sensor(0x00, 1).await.unwrap();

    let mut rc_volt: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc_volt = lock.get_from_kind(IoTypeId::Voltage).await?;
    }    
    let (mut voltage_rx, jh) = rc_volt.enable_16bit_sensor(0x00, 1).await.unwrap();

    // let j = tokio::spawn(async move {
    //     while let Ok(data) = rssi_rx.recv().await {
    //         println!("Rssi: {:?} {:?}", data, data[0] as i8)
    //         // match data {
    //         //     _ => { println!("Hej! Annan knapp");}
                
    //         // }
    //     }
    // });

    let j2 = tokio::spawn(async move {
        while let Ok(data) = remote_a_rx.recv().await {
            println!("Remote_a: {:?}", data)
            // match data {
            //     _ => { println!("Hej! Annan knapp");}
                
            // }
        }
    });

    tokio::spawn(async move {
        while let Ok(data) = voltage_rx.recv().await {
            println!("Voltage: {:?}", data)
            // match data {
            //     _ => { println!("Hej! Annan knapp");}
                
            // }
        }
    });




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