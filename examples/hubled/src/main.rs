// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;
use lego_powered_up::devices::iodevice::IoDevice;
use tokio::time::sleep as tokiosleep;



// use lego_powered_up::{PoweredUp, Hub, HubFilter, devices::Device, error::Error, ConnectedHub};
// use lego_powered_up::notifications::NotificationMessage;
// use lego_powered_up::notifications::NetworkCommand::ConnectionRequest;
// use lego_powered_up::notifications::*;
// use lego_powered_up::consts::*;
// use lego_powered_up::devices::iodevice::IoDevice;

// use lego_powered_up::btleplug::api::{Peripheral, ValueNotification};

// use lego_powered_up::{futures::{future, select, stream::{StreamExt, FuturesUnordered}};



// Powered up
use lego_powered_up::{PoweredUp, Hub, HubFilter, ConnectedHub,}; 

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

use lego_powered_up::consts::IoTypeId;
use lego_powered_up::devices::remote::RcDevice;
use lego_powered_up::devices::light::*;

pub enum Tx {
    Remote()
}



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
    tokiosleep(Duration::from_secs(1)).await;  //Wait for attached devices to be collected


    let hub: ConnectedHub = h.remove(0);
    

    let mut hubled: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hubled = lock.get_from_kind(IoTypeId::HubLed).await?;
        // hubled = lock.get_from_port(0x34).await?;
    }
    // hubled.set_hubled_mode(HubLedMode::Colour).await;
    // hubled.set_hubled_color(Color::Cyan).await;
    hubled.set_hubled_mode(HubLedMode::Rgb).await;
    hubled.set_hubled_rgb(&[0,0,100]).await;


    // Start ui
    let mutex = hub.mutex.clone();
    ui(mutex).await;

    // Cleanup after ui exit
    let lock = hub.mutex.lock().await;
    println!("Disconnect from hub `{}`", lock.name().await?);
    lock.disconnect().await?;
    println!("Done!");

    Ok(())
}

async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
    led.set_rgb(&[red, green, blue]).await
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
