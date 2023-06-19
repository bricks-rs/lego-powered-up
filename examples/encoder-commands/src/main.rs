// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::time::Duration;
use tokio::time::sleep as sleep;

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
    let hub = lego_powered_up::setup::single_hub().await?;

    let motor: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        motor = lock.io_from_port(named_port::C)?;
    }
    let (mut motor_rx, _position_task) = motor
        .enable_32bit_sensor(modes::InternalMotorTacho::POS, 1)
        .await?;

    tokio::spawn(async move {
        while let Ok(data) = motor_rx.recv().await {
            println!(
                "Pos: {:?}",
                data
            );
        }
    });

    // Rotate by degrees (180 cw)
    println!("Rotate by degrees (180 cw)");
    motor.start_speed_for_degrees(180, 50, 50, EndState::Brake).await?;
    sleep(Duration::from_secs(2)).await;

    // Go to position (back to start)
    println!("Go to position (back to start)");
    
    sleep(Duration::from_secs(5)).await;

    // Run for time (hub-controlled)
    println!("Run for time (hub-controlled)");
    motor.start_speed_for_time(5, 50, 50, EndState::Float).await?;
    sleep(Duration::from_secs(10)).await;


    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    Ok(())
}

