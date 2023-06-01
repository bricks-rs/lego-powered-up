// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;
use tokio::time::sleep as tokiosleep;

// Powered up
use lego_powered_up::{PoweredUp, Hub, HubFilter, ConnectedHub, IoDevice}; 

// Access hub 
use std::sync::{Arc};
use tokio::sync::Mutex;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;

// / Access devices
use lego_powered_up::{ error::Error};  //devices::Device,
use tokio::sync::broadcast;
use tokio::sync::mpsc;

// RC
// use lego_powered_up::hubs::remote::*;

// Handle notifications
use core::pin::Pin;
use lego_powered_up::futures::stream::{Stream, StreamExt};
use lego_powered_up::btleplug::api::ValueNotification;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

use lego_powered_up::devices::remote::RcDevice;
use lego_powered_up::devices::remote::RcButtonState;


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
        h.push(ConnectedHub::setup_hub(created_hub).await.expect("Error setting up hub"))
    }

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

    // Set up RC input 
    let rc: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc = lock.io_from_port(0x00).await?;
    }    
    let (mut rc_rx, _) = rc.remote_connect_with_green().await?;

    tokio::spawn(async move {
        while let Ok(data) = rc_rx.recv().await {
            match data {
                RcButtonState::Aup => { println!("A side released") }
                RcButtonState::Aplus => { println!("A plus") }
                RcButtonState::Ared => { println!("A red") }
                RcButtonState::Aminus => { println!("A minus") }
                RcButtonState::Bup => { println!("B side released") }
                RcButtonState::Bplus => { println!("B plus") }
                RcButtonState::Bred => { println!("B red") }
                RcButtonState::Bminus => { println!("B minus") }
                RcButtonState::Green => { println!("Green pressed") }
                RcButtonState::GreenUp => { println!("Green released") }
                _ => ()
            }
        }
    });

    // Set up motor feedback
    // let setup = ConnectedHub::set_up_handler(main_hub.mutex.clone()).await;
    // let motor1 = tokio::spawn(async move { motor_handler(setup.0, setup.1, setup.2).await; });
    // use lego_powered_up::devices::MotorSensorMode;
    // {
    //     let lock = main_hub.mutex.lock().await;
    //     let mut motor_c = lock.enable_from_port(0x02).await?;
    //     motor_c.motor_sensor_enable(MotorSensorMode::Pos, 1).await?;
    // }



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
