// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use lego_powered_up::{IoTypeId, IoDevice,
                    consts,
                    devices::light::{self, HubLed},
                    devices::motor::{EncoderMotor, Power} 
                };
use core::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = lego_powered_up::setup::single_hub().await?;

    // Devices to be used
    let hub_led: IoDevice;
    let motor_c: IoDevice;
    let motor_d: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hub_led = lock.io_from_kind(IoTypeId::HubLed).await?;
        motor_c = lock.io_from_port(consts::named_port::C).await?; 
        motor_d = lock.io_from_port(consts::named_port::D).await?; 
    }    
    
    println!("Change the hub LED to green");
    hub_led.set_hubled_mode(light::HubLedMode::Colour).await?;
    hub_led.set_hubled_color(consts::Color::Green).await?;
   
    println!("Run motors");
    motor_c.start_speed(50, Power::Cw(50)).await?;
    motor_d.start_speed(50, Power::Cw(50)).await?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Stop motors");
    motor_c.start_power(Power::Float).await?;
    motor_d.start_power(Power::Brake).await?;

    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("Done!");

    Ok(())
}
