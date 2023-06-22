// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::time::Duration;
use tokio::time::sleep;

use lego_powered_up::consts::named_port;
use lego_powered_up::consts::LEGO_COLORS;
use lego_powered_up::error::{Error, OptionContext, Result};
use lego_powered_up::iodevice::modes;
use lego_powered_up::iodevice::remote::{RcButtonState, RcDevice};
use lego_powered_up::iodevice::{hubled::*, motor::*, sensor::*};
use lego_powered_up::notifications::Power;
use lego_powered_up::{ConnectedHub, IoDevice, IoTypeId, PoweredUp};
use lego_powered_up::{Hub, HubFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === Single hub ===
    let hub = lego_powered_up::setup::single_hub().await?;

    // Do stuff

    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    // === Main hub and RC ===
    let (main_hub, rc_hub) = lego_powered_up::setup::main_and_rc().await?;
    let rc: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc = lock.io_from_port(named_port::A)?;
    }
    let (mut rc_rx, _) = rc.remote_connect_with_green()?;

    // Do stuff

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
