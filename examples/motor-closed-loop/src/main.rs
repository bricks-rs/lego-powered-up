// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::pin::Pin;
use core::time::Duration;
use futures::stream::{Stream, StreamExt};

use std::sync::{Arc};
use tokio::time::sleep as tokiosleep;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
// use tokio::sync::mpsc;
use btleplug::api::ValueNotification;

use lego_powered_up::{PoweredUp, Hub, HubFilter, ConnectedHub, IoDevice}; 
use lego_powered_up::{ error::Error};  
use lego_powered_up::consts::MotorSensorMode;
use lego_powered_up::consts::named_port;
use lego_powered_up::devices::motor::EncoderMotor;
use lego_powered_up::devices::remote::RcDevice;
use lego_powered_up::devices::remote::RcButtonState;
use lego_powered_up::devices::sensor::GenericSensor;
use lego_powered_up::devices::modes;
use lego_powered_up::notifications::Power;

type HubMutex = Arc<Mutex<Box<dyn Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init PoweredUp with found adapter
    println!("Looking for BT adapter and initializing PoweredUp with found adapter");
    let mut pu = PoweredUp::init().await?;

    let hub_count = 2;
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
    let main_hub: ConnectedHub = h.remove(0);
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
        rc = lock.io_from_port(named_port::A).await?;
    }    
    let (mut rc_rx, _) = rc.remote_connect_with_green().await?;

    // Set up motor feedback
    let motor: IoDevice;
    {
        let lock = main_hub.mutex.lock().await;
        motor = lock.io_from_port(named_port::A).await?;
    }
    motor.motor_sensor_enable(MotorSensorMode::Pos, 1).await?;
    let (mut motor_rx, _) = motor.enable_32bit_sensor(modes::InternalMotorTacho::POS, 1).await?;

    let motor_control = tokio::spawn(async move {
        const MAX_POWER: Power = Power::Cw((100));
        let mut set_limit: (Option<i32>, Option<i32>) = (None, None);
        let mut set_speed: i8 = 20;
        let mut pos: i32 = 0;
        let mut at_limit: (bool, bool) = (false, false);
        let mut cmd: (bool, bool) = (false, false);
        loop {
            tokio::select! {
                Ok(msg) = rc_rx.recv() => {
                    match msg {
                        RcButtonState::Aup => { 
                            motor.start_power(Power::Brake).await;
                            cmd = (false, false);
                        }
                        RcButtonState::Aminus => { 
                            if !at_limit.0 {
                                cmd.0 = true;
                                motor.start_speed(-set_speed, MAX_POWER).await;
                            }
                        }
                        RcButtonState::Aplus => { 
                            if !at_limit.1 { 
                                cmd.1 = true;
                                motor.start_speed(set_speed, MAX_POWER).await; 
                            }
                        }
                        RcButtonState::Ared => { 
                            match set_limit.0 {
                                None => {
                                    set_limit.0 = Some(pos);
                                    println!("Left limit set: {:?}", pos);
                                }
                                Some(_) => { 
                                    set_limit.0 = None;
                                    println!("Left limit cancelled");
                                } 
                            }
                        }
                        RcButtonState::Bred => { 
                            match set_limit.1 {
                                None => {
                                    set_limit.1 = Some(pos);
                                    println!("Right limit set: {:?}", pos);
                                }
                                Some(_) => { 
                                    set_limit.1 = None;
                                    println!("Right limit cancelled");
                                }
                            }
                        }
                        RcButtonState::Bplus => { 
                            if set_speed < 96 { 
                                set_speed += 5; 
                                println!("Set speed: {}", set_speed);
                            }
                        }
                        RcButtonState::Bminus => { 
                            if set_speed > 4 { 
                                set_speed -= 5; 
                                println!("Set speed: {}", set_speed);
                            } 
                        }

                        // RcButtonState::Bup => { println!("B side released"); }
                        RcButtonState::Green => { println!("Green pressed");
                            println!("Exiting");
                            break;
                        }
                        // RcButtonState::GreenUp => { println!("Green released") }
                        _ => ()
                    }
                }
                Ok(msg) = motor_rx.recv() => {
                    pos = msg[0];
                    match set_limit.0 {
                        None => (),
                        Some(limit) => {
                            if (pos <= limit) & cmd.0 {
                                motor.start_power(Power::Brake).await;
                                at_limit.0 = true;
                                println!("Left LIMIT: {}", limit);
                            } else {
                                at_limit.0 = false;
                            }
                        }
                    }
                    match set_limit.1 {
                        None => (),
                        Some(limit) => {
                            if (pos >= limit) & cmd.1 {
                                motor.start_power(Power::Brake).await;
                                at_limit.1 = true;
                                println!("Right LIMIT: {}", limit);
                            } else {
                                at_limit.1 = false;
                            }

                        }
                    }
                    println!("Pos: {}", pos);
                }
                else => { break }    
            };
        }    
    });

    motor_control.await;

    // Cleanup 
    println!("Disconnect from hub `{}`", rc_hub.name);
    {
        let lock = rc_hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("Disconnect from hub `{}`", main_hub.name);
    {
        let lock = main_hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    
    println!("Done!");

    Ok(())
}
