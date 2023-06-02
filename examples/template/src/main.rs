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
use lego_powered_up::devices::modes;
use lego_powered_up::devices::remote::{RcDevice, RcButtonState};
use lego_powered_up::devices::{light::*, sensor::*, motor::*};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === Single hub === 
    let hub = setup::single_hub().await?;

    // Do stuff

    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }


    // === Main hub and RC ===
    let (main_hub, rc_hub) = setup::main_and_rc().await?;
    let rc: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc = lock.io_from_port(named_port::A).await?;
    }    
    let (mut rc_rx, _) = rc.remote_connect_with_green().await?;

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

