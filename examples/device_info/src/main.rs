// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;
use tokio::time::sleep as tokiosleep;

use lego_powered_up::{PoweredUp, HubFilter, ConnectedHub};

// use lego_powered_up::{PoweredUp, Hub, HubFilter, devices::Device, error::Error, ConnectedHub};
// use lego_powered_up::notifications::NotificationMessage;
// use lego_powered_up::notifications::NetworkCommand::ConnectionRequest;
// use lego_powered_up::notifications::*;
// use lego_powered_up::consts::*;
// use lego_powered_up::devices::iodevice::IoDevice;


// Btleplug reexports
// use lego_powered_up::btleplug::api::{Peripheral, ValueNotification};

// Futures reexports
// use lego_powered_up::futures::stream::Stream;
// use lego_powered_up::{futures::{future, select, stream::{StreamExt, FuturesUnordered}};

// use core::pin::Pin;
// use std::sync::{Arc};
// use tokio::sync::Mutex;



// #[macro_use] 
use text_io::*;

// type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
// type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init PoweredUp with found adapter
    println!("Looking for BT adapter and initializing PoweredUp with found adapter");
    let mut pu = PoweredUp::init().await?;

    let hub_count = 1;
    println!("Waiting for hubs...");
    let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, &hub_count).await?;
    
    println!("Discovered {} hubs, trying to connect...", &hub_count);
    let mut dh_iter = discovered_hubs.into_iter();
    let dh1 = dh_iter.next().unwrap();
    println!("Connecting to hub `{}`", dh1.name);

    // Setup hub
    let created_hub = pu.create_hub(&dh1).await?;
    let hub1 = ConnectedHub::setup_hub(created_hub).await;

    {
        // tokiosleep(Duration::from_secs(5)).await;
        // println!("req name for port:{} mode:{}", 1, 0);
        // let mut lock = hub1.mutex.lock().await;
        // lock.request_mode_info(1, 0, ModeInformationType::Name);
    }

    {
        // tokiosleep(Duration::from_secs(3)).await;
        // let mut lock = hub1.mutex.lock().await;
        // // dbg!(lock.connected_io());
        // for device in lock.connected_io().values() {
        //     println!("{}", device);
        // }
        // tokiosleep(Duration::from_secs(5)).await;

    }

    loop {
        print!("(l)ist, <port> or (q)uit > ");
        let line: String = read!("{}\n");
        if line.len() == 1 {
            continue;
        } 
        else if line.contains("l") {
            let mut lock = hub1.mutex.lock().await;
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
            match input {
                Ok(num) => {
                    // println!("Number: {}", num);
                    let mut lock = hub1.mutex.lock().await;
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


    let lock = hub1.mutex.lock().await;
    println!("Disconnect from hub `{}`", lock.name().await?);
    lock.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

// async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
//     led.set_rgb(&[red, green, blue]).await
// }


