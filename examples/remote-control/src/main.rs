// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

// #![allow(unused)]

use lego_powered_up::{PoweredUp, HubFilter};
use lego_powered_up::notifications::NotificationMessage;
// Btleplug reexports
use lego_powered_up::btleplug::api::{Central, Peripheral};
// Futures reexports
use lego_powered_up::futures::stream::StreamExt;


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
    
    let mut hub1_stream =
                        hub1.peripheral().notifications().await?;  


    // let hub1_name = hub1.name().await?;

    // Track button status
    let mut remote_status = lego_powered_up::hubs::remote::RemoteStatus::new();


    tokio::spawn(async move {
        while let Some(data) = hub1_stream.next().await {
            // println!("Received data from {:?} [{:?}]: {:?}", &hub1_name, data.uuid, data.value);

            let r = NotificationMessage::parse(&data.value);
            match r {
                Ok(n) => {
                    // println!("{}", &hub1_name);
                    // dbg!(&n);
                    match n {
                        NotificationMessage::PortValueSingle(val) => {
                            match val.values[0] {
                                0x0 => {
                                    match val.values[1] {
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
                                    dbg!(remote_status);
                                }
                                0x1 => {
                                    match val.values[1] {
                                        0 => {
                                            remote_status.b_plus = false;
                                            remote_status.b_red = false; 
                                            remote_status.b_minus = false;
                                            println!("Some B-side button released");
                                        }
                                        1 => {
                                            remote_status.b_plus = true;
                                            println!("B+ pressed");
                                        }
                                        127 => {
                                            remote_status.b_red = true; 
                                            println!("Bred pressed");
                                        }
                                        255 => {
                                            remote_status.b_minus = true;
                                            println!("B- pressed");
                                        }
                                        _  => ()
                                    }
                                    dbg!(remote_status);
                                }
                                _ => ()                                
                            }
                            
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

    
    println!("Enable notifcations for A-side buttons, mode 1");
    let mut remote_a = hub1.port(lego_powered_up::hubs::Port::A).await?;
    remote_a.remote_buttons_enable(1, 1).await?;

    println!("Enable notifcations for B-side buttons, mode 1");
    let mut remote_b = hub1.port(lego_powered_up::hubs::Port::B).await?;
    remote_b.remote_buttons_enable(1, 1).await?;


    loop {}

    println!("Disconnect from hub `{}`", hub1.name().await?);
    hub1.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

