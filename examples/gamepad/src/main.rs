// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;

use lego_powered_up::{PoweredUp, HubFilter, devices::Device, error::Error};
use lego_powered_up::notifications::NotificationMessage;
use lego_powered_up::notifications::NetworkCommand::ConnectionRequest;
// Btleplug reexports
use lego_powered_up::btleplug::api::{Central, Peripheral};
use lego_powered_up::btleplug::api::ValueNotification;

// Futures reexports
use lego_powered_up::{futures::stream::{StreamExt, FuturesUnordered, Stream}};
use lego_powered_up::{futures::{future, select}};

use core::pin::Pin;





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

    let hub_count = 1;
    println!("Waiting for hubs...");
    let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, &hub_count).await?;
    
    println!("Discovered {} hubs, trying to connect...", &hub_count);
    let mut dh_iter = discovered_hubs.into_iter();
    let dh1 = dh_iter.next().unwrap();
    println!("Connecting to hub `{}`", dh1.name);
    let hub1 = pu.create_hub(&dh1).await?;
    // dbg!(hub1.properties());
    
    let s: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;
    let mut hub1_stream =
                        hub1.peripheral().notifications().await?;  


    let hub1_name = hub1.name().await?;

    // Track button status
    let mut remote_status = lego_powered_up::hubs::remote::RemoteStatus::new();

    println!("Enable notifcations for A-side buttons, mode 1");
    let mut remote_a = hub1.port(lego_powered_up::hubs::Port::A).await?;
    remote_a.remote_buttons_enable(1, 1).await?;

    println!("Enable notifcations for B-side buttons, mode 1");
    let mut remote_b = hub1.port(lego_powered_up::hubs::Port::B).await?;
    remote_b.remote_buttons_enable(1, 1).await?;

    println!("Connect hub LED");
    let mut hub_led = hub1.port(lego_powered_up::hubs::Port::HubLed).await?;
    hub_led.set_rgb(&[0x00, 0x00, 0x00]).await?;


    // tokio::spawn(async move {
    //     while let Some(data) = hub1_stream.next().await {
    //         println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);

    //         let r = NotificationMessage::parse(&data.value);
    //         match r {
    //             Ok(n) => {
    //                 println!("{}", &hub1_name);
    //                 dbg!(&n);
    //             }
    //             Err(e) => {
    //                 println!("Parse error: {}", e);
    //             }
    //         }

    //     }  
    // });

    tokio::spawn(async move {
        while let Some(data) = hub1_stream.next().await {
            // println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);

            let r = NotificationMessage::parse(&data.value);
            match r {
                Ok(n) => {
                    // println!("{}", &hub1_name);
                    // dbg!(&n);
                    match n {
                        NotificationMessage::HwNetworkCommands(cmd) => {
                            match cmd {
                                ConnectionRequest(state) => {
                                    match state {
                                        lego_powered_up::notifications::ButtonState::Up => {
                                            remote_status.green = true;
                                            println!("Green pressed");
                                        }
                                        lego_powered_up::notifications::ButtonState::Released => {
                                            remote_status.green = false;
                                            println!("Green released");
                                        }
                                        _ => ()
                                    }    
                                }
                                _ => ()
                            }
                        }
                        NotificationMessage::PortValueSingle(val) => {
                            match val.port_id {
                                0x0 => {
                                    match val.data[0] {
                                        0 => {
                                            remote_status.a_plus = false;
                                            remote_status.a_red = false; 
                                            remote_status.a_minus = false;
                                            println!("Some A-side button released");
                                        }
                                        1 => {
                                            remote_status.a_plus = true;
                                            println!("A+ pressed");
                                        }
                                        127 => {
                                            remote_status.a_red = true; 
                                            println!("Ared pressed");
                                        }
                                        255 => {
                                            remote_status.a_minus = true;
                                            println!("A- pressed");
                                        }
                                        _  => ()
                                    }
                                }
                                0x1 => {
                                    match val.data[0] {
                                        0 => {
                                            remote_status.b_plus = false;
                                            remote_status.b_red = false; 
                                            remote_status.b_minus = false;
                                            println!("Some B-side button released");
                                            hub_led.set_rgb(&[0x00, 0x00, 0x00]).await.unwrap();
                                        }
                                        1 => {
                                            remote_status.b_plus = true;
                                            println!("B+ pressed");
                                            hub_led.set_rgb(&[0x00, 0xff, 0x00]).await.unwrap();
                                        }
                                        127 => {
                                            remote_status.b_red = true; 
                                            println!("Bred pressed");
                                            // set_led(*hub_led, 0xFF, 0x00, 0x00);
                                            hub_led.set_rgb(&[0xff, 0x00, 0x00]).await.unwrap();
                                        }
                                        255 => {
                                            remote_status.b_minus = true;
                                            println!("B- pressed");
                                            hub_led.set_rgb(&[0x00, 0x00, 0xff]).await.unwrap();
                                        }
                                        _  => ()
                                    }
                                    
                                }
                                _ => ()                                
                            }
                            // dbg!(remote_status);
                        }
                        _ => ()
                    }
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                }
            }
        }  
    });


    // while !remote_status.a_red && !remote_status.b_red {
        loop {
            dbg!(remote_status);
            if remote_status.a_red && remote_status.b_red {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    // }

    println!("Disconnect from hub `{}`", hub1.name().await?);
    hub1.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
    led.set_rgb(&[red, green, blue]).await
}