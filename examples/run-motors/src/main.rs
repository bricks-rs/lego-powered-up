// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use core::time::Duration;
use lego_powered_up::{
    consts,
    iodevice::hubled::{self, HubLed},
    iodevice::motor::{EncoderMotor, Power},
    IoDevice, IoTypeId,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = lego_powered_up::setup::single_hub().await?;

    // Devices to be used
    let hub_led: IoDevice;
    let motor_a: IoDevice;
    let motor_b: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hub_led = lock.io_from_kind(IoTypeId::HubLed)?;
        motor_a = lock.io_from_port(consts::named_port::A)?;
        motor_b = lock.io_from_port(consts::named_port::B)?;
    }

    println!("Change the hub LED to green");
    hub_led.set_hubled_mode(hubled::HubLedMode::Colour).await?;
    hub_led.set_hubled_color(consts::Color::Green).await?;

    println!("Run motors");
    motor_a.start_speed(50, 50).await?;
    motor_b.start_speed(50, 50).await?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Stop motors");
    motor_a.start_power(Power::Float).await?;
    motor_b.start_power(Power::Brake).await?;

    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("Done!");

    Ok(())
}
