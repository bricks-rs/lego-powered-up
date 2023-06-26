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
    let hub = lego_powered_up::setup::single_hub().await?;
    let motor = hub.mutex.lock().await.io_from_port(named_port::A)?;

    // {
    //     println!("Speed run");
    //     let (mut rx, _task) = 
    //         motor.enable_8bit_sensor(modes::InternalMotorTacho::SPEED, 1)?;
    //     let sensor_task = tokio::spawn(async move {
    //         while let Ok(data) = rx.recv().await {
    //             println!("Speed: {:?}", data);
    //         }
    //     });
    //     motor.preset_encoder(0);   
    //     motor.start_speed_for_degrees(45, 20, 50, EndState::Brake);
    //     sleep(Duration::from_secs(1)).await;
    //     motor.goto_absolute_position(0, 20, 50, EndState::Brake);
    //     sleep(Duration::from_secs(3)).await;
    // }

    // {
    //     println!("\nPosition run");
    //     let (mut rx, _task) = 
    //         motor.enable_32bit_sensor(modes::InternalMotorTacho::POS, 1)?;
    //     let sensor_task = tokio::spawn(async move {
    //         while let Ok(data) = rx.recv().await {
    //             println!("Position: {:?}", data);
    //         }
    //     });
    //     motor.preset_encoder(0);   
    //     motor.start_speed_for_degrees(45, 20, 50, EndState::Brake);
    //     sleep(Duration::from_secs(1)).await;
    //     motor.goto_absolute_position(0, 20, 50, EndState::Brake);
    //     sleep(Duration::from_secs(3)).await;
    // }

    {
        println!("\nCombined run");
        let (mut rx, _task) = 
        motor.motor_combined_sensor_enable( 1, 1).await?;
        let sensor_task = tokio::spawn(async move {
            while let Ok(data) = rx.recv().await {
                println!("Combined: {:?}", data);
            }
        });
        sleep(Duration::from_secs(2)).await;
        motor.preset_encoder(0);   
        sleep(Duration::from_secs(1)).await;
        motor.start_speed_for_degrees(45, 20, 50, EndState::Brake);
        sleep(Duration::from_secs(1)).await;
        motor.goto_absolute_position(0, 20, 50, EndState::Brake);
        sleep(Duration::from_secs(1)).await;
        motor.preset_encoder(0);   
        sleep(Duration::from_secs(1)).await;
        motor.start_speed_for_degrees(180, 50, 50, EndState::Brake);
        sleep(Duration::from_secs(2)).await;
        motor.goto_absolute_position(360, 20, 50, EndState::Brake);
        sleep(Duration::from_secs(2)).await;
        motor.goto_absolute_position(0, 20, 50, EndState::Brake);
        sleep(Duration::from_secs(2)).await;
    }
    
    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    hub.mutex.lock().await.disconnect().await?;

    Ok(())
}