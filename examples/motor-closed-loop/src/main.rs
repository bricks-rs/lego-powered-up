// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;
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
use lego_powered_up::NotificationHandler;

// Access hub 
use std::sync::{Arc};
use tokio::sync::Mutex;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type HandlerMutex = Arc<Mutex<Box<dyn NotificationHandler>>>;

// / Access devices
use lego_powered_up::{devices::Device, error::Error};
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
        h.push(ConnectedHub::setup_hub(created_hub).await)
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
    let setup = ConnectedHub::set_up_handler(rc_hub.mutex.clone()).await;
    let (rc_tx, mut rc_rx) = broadcast::channel::<RcButtonState>(3);
    let rc_tx_clone = rc_tx.clone();
    // let remote_handler1 = tokio::spawn(async move { 
    //     rc_handler(setup.0, setup.1, setup.2, rc_tx_clone).await; 
    // }); 
    {
        let lock = rc_hub.mutex.lock().await;
        let mut remote_a = lock.get_from_port(0x00).await?;
        remote_a.remote_buttons_enable(1, 1).await?;
        let mut remote_b = lock.get_from_port(0x01).await?;
        remote_b.remote_buttons_enable(1, 1).await?;    // mode 0x1, delta 1
    }    

    let mut rc_rx_test = rc_tx.subscribe();
    let j = tokio::spawn(async move {
        while let Ok(data) = rc_rx_test.recv().await {
            match data {
                RcButtonState::Aup => { println!("Hej! Aup") }
                RcButtonState::Aplus => { println!("Hej! Aplus") }
                RcButtonState::Ared => { println!("Hej! Ared") }
                RcButtonState::Aminus => { println!("Hej! Aminus") }
                _ => { println!("Hej! Annan knapp");}
                
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

async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
    led.set_rgb(&[red, green, blue]).await
}



pub async fn motor_handler(mut stream: PinnedStream, mutex: HubMutex, hub_name: String) {
    use lego_powered_up::notifications::NotificationMessage;

    while let Some(data) = stream.next().await {
        // println!("Received data from {:?} [{:?}]: {:?}", hub_name, data.uuid, data.value);

        let r = NotificationMessage::parse(&data.value);
        match r {
            Ok(n) => {
                // dbg!(&n);
                match n {
                    // Active feedback
                    NotificationMessage::PortValueSingle(val) => {

                    }
                    NotificationMessage::PortValueCombined(val) => {}

                    // Setup feedback and errors
                    NotificationMessage::PortInputFormatSingle(val) => {}
                    NotificationMessage::PortInputFormatCombinedmode(val) => {}
                    NotificationMessage::PortOutputCommandFeedback(val ) => {}
                    NotificationMessage::GenericErrorMessages(val) => {}

                    _ => ()
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

    }  
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
