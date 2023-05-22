// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use lego_powered_up::{notifications::Power, PoweredUp};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Listening for hubs...");
    let mut pu = PoweredUp::init().await?;
    let hub = pu.wait_for_hub().await?;

    println!("Connecting to hub `{}`", hub.name);
    let hub = pu.create_hub(&hub).await?;

    println!("Change the hub LED to green");
    let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed).await?;
    hub_led.set_rgb(&[0, 0xff, 0]).await?;

    println!("Run motors");
    let mut motor_b = hub.port(lego_powered_up::hubs::Port::B).await?;
    motor_b.start_speed(50, Power::Cw(50)).await?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Stop motors");
    motor_b.start_speed(0, Power::Float).await?;

    println!("Disconnect from hub `{}`", hub.name().await?);
    hub.disconnect().await?;

    println!("Done!");

    Ok(())
}
