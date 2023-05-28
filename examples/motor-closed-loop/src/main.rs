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

// Access hub 
use std::sync::{Arc};
use tokio::sync::Mutex;
type HubMutex = Arc<Mutex<Box<dyn Hub>>>;

// / Access devices
use lego_powered_up::{devices::Device, error::Error};

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
    let mut dh_iter = discovered_hubs.into_iter();
    let dh1 = dh_iter.next().unwrap();
    println!("Connecting to hub `{}`", dh1.name);

    // Setup hub
    let created_hub = pu.create_hub(&dh1).await?;
    let hub1 = ConnectedHub::setup_hub(created_hub).await;


    // Set up RC input 
    let setup = ConnectedHub::set_up_handler(hub1.mutex.clone()).await;
    let remote1 = tokio::spawn(async move { remote_control(setup.0, setup.1, setup.2).await; }); 
    

    // Set up motor feedback
    let setup = ConnectedHub::set_up_handler(hub1.mutex.clone()).await;
    let motor1 = tokio::spawn(async move { motor_control(setup.0, setup.1, setup.2).await; });

    // Attempt with function pointer
    // let ptr: fn(stream: PinnedStream, mutex: HubMutex, hub_name: String) -> () = another_handler;
    // let new_handler = ConnectedHub::spawn_handler(hub1.mutex.clone()).await, ptr);

    // Start ui
    let mutex = hub1.mutex.clone();
    ui(mutex).await;

    // Cleanup after ui exit
    let lock = hub1.mutex.lock().await;
    println!("Disconnect from hub `{}`", lock.name().await?);
    lock.disconnect().await?;
    println!("Done!");

    Ok(())
}

async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
    led.set_rgb(&[red, green, blue]).await
}

pub async fn remote_control(mut stream: PinnedStream, mutex: HubMutex, hub_name: String) {
    // use lego_powered_up::notifications::NotificationMessage::*;
    use lego_powered_up::notifications::*;
    use lego_powered_up::notifications::NetworkCommand::ConnectionRequest;
    while let Some(data) = stream.next().await {
        // println!("Received data from {:?} [{:?}]: {:?}", hub_name, data.uuid, data.value);

        let r = NotificationMessage::parse(&data.value);
        match r {
            Ok(n) => {
                // dbg!(&n);
                match n {
                    // Active feedback
                    NotificationMessage::HwNetworkCommands(cmd) => {
                        match cmd {
                            ConnectionRequest(state) => {
                                match state {
                                    ButtonState::Up => {        //Green on
                                    }
                                    ButtonState::Released => {  //Green off
                                    }
                                    _ => ()
                                }    
                            }
                            
                            _ => ()
                        }
                    }
                    NotificationMessage::PortValueSingle(val) => {
                        match val.values[0] {
                            0x0 => {
                                match val.values[1] {
                                    0 => {      // A side off
                                        println!("Some A-side button released");
                                    }
                                    1 => {      // A + on
                                        println!("A+ on");
                                    }
                                    127 => {    // A red on
                                        println!("A red on");
                                    }
                                    255 => {    // A - on
                                        println!("A- on");
                                    }
                                    _  => ()
                                }
                            }
                            0x1 => {
                                match val.values[1] {
                                    0 => {      // B side off
                                        println!("Some B-side button released");
                                    }
                                    1 => {      // B + on
                                        println!("B+ on");
                                    }
                                    127 => {    // B red on
                                        println!("B red on");
                                    }
                                    255 => {    // A + on
                                        println!("B- on");
                                    }
                                    _  => ()
                                }
                                
                            }
                            _ => ()                                
                        }
                        // dbg!(remote_status);
                    }
                    NotificationMessage::PortValueCombinedmode(val) => {}

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

pub async fn motor_control(mut stream: PinnedStream, mutex: HubMutex, hub_name: String) {
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
                    NotificationMessage::PortValueCombinedmode(val) => {}

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
