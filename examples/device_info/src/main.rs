// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use std::time::Duration;

use lego_powered_up::hubs::ConnectedIo;
use lego_powered_up::{PoweredUp, Hub, HubFilter, devices::Device, error::Error};
use lego_powered_up::notifications::NotificationMessage;
use lego_powered_up::notifications::NetworkCommand::ConnectionRequest;
use lego_powered_up::notifications::*;
use lego_powered_up::consts::*;

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

    let hub1 = pu.create_hub(&dh1).await?;
    let hub1_name = hub1.name().await?;
    let mut hub1_stream: PinnedStream = hub1.peripheral().notifications().await?;  

    let mu_hub1: HubMutex = Arc::new(Mutex::new(hub1));
    
    let new_handle = mu_hub1.clone();
    tokio::spawn(async move {
        process(hub1_stream, new_handle, &hub1_name).await;
    });


    {
        let a_hub = mu_hub1.lock().await;
        a_hub.request_port_info(0x0, InformationType::ModeInfo).await?;
        // hub1.request_mode_info(port_id, mode, infotype);
    }

    // loop {
    //     let a_hub = mu_hub1.lock().await;
    //     dbg!(a_hub.attached_io_raw());
    //     tokio::time::sleep(Duration::from_secs(5)).await;
    // }

    {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let a_hub = mu_hub1.lock().await;
        dbg!(a_hub.attached_io_raw());
        // tokio::time::sleep(Duration::from_secs(5)).await;

    }

    let a_hub = mu_hub1.lock().await;
    println!("Disconnect from hub `{}`", a_hub.name().await?);
    a_hub.disconnect().await?;
    
    println!("Done!");

    Ok(())
}

async fn set_led(mut led: Box<dyn Device>, red: u8, green: u8, blue: u8) -> Result<(), Error> {
    led.set_rgb(&[red, green, blue]).await
}

async fn process(mut stream: PinnedStream, m_hub: HubMutex, hub_name: &str) {
    while let Some(data) = stream.next().await {
        
        println!("Received data from {:?} [{:?}]: {:?}", hub_name, data.uuid, data.value);

        

        let r = NotificationMessage::parse(&data.value);
        match r {
            Ok(n) => {
                // println!("{}", hub_name);
                // dbg!(&n);
                match n {
                    NotificationMessage::PortInformation(val) => {


                    }
                    NotificationMessage::HubAttachedIo(io_event) => {
                        match io_event {
                            AttachedIo{port, event} => {
                                let port_id = port;
                                match event {
                                    IoAttachEvent::AttachedIo{io_type_id, hw_rev, fw_rev} => {
                                        let aio = ConnectedIo {
                                            port_id: port_id,
                                            io_type_id,
                                        };
                                        {
                                            let mut hub = m_hub.lock().await;
                                            hub.attach_io(aio);
                                        }
                                    }
                                    IoAttachEvent::DetachedIo{} => {}
                                    IoAttachEvent::AttachedVirtualIo {port_a, port_b }=> {}
                                }

                            }
                        }

                    }
                    _ => ()
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

    }  
}