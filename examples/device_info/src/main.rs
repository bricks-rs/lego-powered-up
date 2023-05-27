// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;

use lego_powered_up::{PoweredUp, Hub, HubFilter, devices::Device, error::Error};
use lego_powered_up::notifications::NotificationMessage;
use lego_powered_up::notifications::NetworkCommand::ConnectionRequest;
use lego_powered_up::notifications::*;
use lego_powered_up::consts::*;
use lego_powered_up::devices::iodevice::IoDevice;


// Btleplug reexports
use lego_powered_up::btleplug::api::{Central, Peripheral};
use lego_powered_up::btleplug::api::ValueNotification;

// Futures reexports
use lego_powered_up::{futures::stream::{StreamExt, FuturesUnordered, Stream}};
use lego_powered_up::{futures::{future, select}};


use core::pin::Pin;


use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;


type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

// use lego_powered_up::
struct ConnectedHub {
    name: String,
    mutex: HubMutex
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
    let mut dh_iter = discovered_hubs.into_iter();
    let dh1 = dh_iter.next().unwrap();
    println!("Connecting to hub `{}`", dh1.name);

    // Setup hub
    let created_hub = pu.create_hub(&dh1).await?;
    let mut created_stream: PinnedStream = created_hub.peripheral().notifications().await?;  
    let hub1 = ConnectedHub {
        name: created_hub.name().await?,
        mutex: Arc::new(Mutex::new(created_hub)),
    };
    let mutex_handle = hub1.mutex.clone();
    tokio::spawn(async move {
        lego_powered_up::hubs::parse_notification_stream(created_stream, mutex_handle, &hub1.name).await;
    });
    {
        let mut lock = hub1.mutex.lock().await;
        lock.peripheral().subscribe(&lock.characteristic()).await?;
    }
    // Setup done

    {
        tokio::time::sleep(Duration::from_secs(5)).await;
        // println!("req name for port:{} mode:{}", 1, 0);
        // let mut lock = hub1.mutex.lock().await;
        // lock.request_mode_info(1, 0, ModeInformationType::Name);
    }

    {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let mut lock = hub1.mutex.lock().await;
        dbg!(lock.connected_io());
        // tokio::time::sleep(Duration::from_secs(5)).await;

    }


    let a_hub = hub1.mutex.lock().await;
    println!("Disconnect from hub `{}`", a_hub.name().await?);
    a_hub.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
    led.set_rgb(&[red, green, blue]).await
}

