// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::time::Duration;
use tokio::time::sleep as sleep;

use lego_powered_up::setup;
use lego_powered_up::{PoweredUp, ConnectedHub, IoDevice, IoTypeId}; 
use lego_powered_up::{Hub, HubFilter, }; 
use lego_powered_up::error::{Error, Result, OptionContext}; 
use lego_powered_up::consts::named_port;
use lego_powered_up::notifications::Power;
use lego_powered_up::consts::{LEGO_COLORS, };
use lego_powered_up::iodevice::modes;
use lego_powered_up::iodevice::remote::{RcDevice, RcButtonState};
use lego_powered_up::iodevice::{hubled::*, sensor::*, motor::*};
// use lego_powered_up::iodevice::{notifications::EndState};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = setup::single_hub().await?;
    const MAX_POWER: Power = Power::Cw(100);

    let motor_a: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        motor_a = lock.io_from_port(named_port::A).await?;
    }
    let (mut motor_a_rx, _position_task) = motor_a.enable_32bit_sensor(modes::InternalMotorTacho::POS, 1).await?;
    motor_a.start_speed_for_degrees(90, 30, MAX_POWER, EndState::Float ).await?;
    sleep(Duration::from_millis(2000)).await;

    let motor_b: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        motor_b = lock.io_from_port(named_port::B).await?;
    }
    let (mut motor_b_rx, _position_task) = motor_b.enable_32bit_sensor(modes::InternalMotorTacho::POS, 1).await?;
    motor_b.start_speed_for_degrees(90, 30, MAX_POWER, EndState::Float ).await?;
    sleep(Duration::from_millis(2000)).await;


    let motor_ab: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        motor_ab = lock.io_from_port(named_port::MOVE_AB).await?;
    }
    let (mut motor_ab_rx, _position_task) = motor_ab.enable_32bit_sensor(modes::InternalMotorTacho::POS, 1).await?;
    motor_ab.start_speed_for_degrees(90, 30, MAX_POWER, EndState::Float ).await?;
    sleep(Duration::from_millis(2000)).await;



    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }


    Ok(())
}


// let rc_control = tokio::spawn(async move {
//     while let Ok(data) = rc_rx.recv().await {
//         match data {
//             RcButtonState::Aup => {  println!("A released"); }
//             RcButtonState::Aplus => { println!("A plus") }
//             RcButtonState::Ared => { println!("A red"); }
//             RcButtonState::Aminus => { println!("A minus") }
//             RcButtonState::Bup => { println!("B released"); 
//             RcButtonState::Bplus => { println!("B plus") }
//             RcButtonState::Bred => { println!("B red");  }
//             RcButtonState::Bminus => { println!("B minus") }
//             RcButtonState::Green => { println!("Green pressed") }
//             RcButtonState::GreenUp => { println!("Green released") }
//         }
//     }
// });