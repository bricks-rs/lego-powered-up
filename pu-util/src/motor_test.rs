// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::argparse::MotorTestArgs;
use anyhow::Result;
use lego_powered_up::{
    consts,
    iodevice::hubled::HubLed,
    iodevice::motor::{EncoderMotor, Power},
    ConnectedHub, IoDevice, IoTypeId,
};
use lego_powered_up::{HubFilter, PoweredUp};

use std::time::Duration;

pub async fn run(args: &MotorTestArgs) -> Result<()> {
    let mut pu = if let Some(dev) = args.device_index {
        PoweredUp::with_device_index(dev).await?
    } else {
        PoweredUp::init().await?
    };
    // let rx = pu.event_receiver().unwrap();
    // pu.run().unwrap();

    println!("Listening for hub announcements...");

    // 90:84:2B:60:3C:B8
    // 90:84:2B:60:3A:6C

    // let hub = DiscoveredHub {
    //     hub_type: HubType::Unknown,
    //     addr: BDAddr::from_str(&args.address)?,
    //     name: "".to_string(),
    // };

    let hub = pu
        .wait_for_hub_filter(if let Some(addr) = &args.address {
            HubFilter::Addr(addr.to_string())
        } else {
            HubFilter::Null
        })
        .await?;

    println!(
        "Connecting to `{}` `{}` with address `{}`",
        hub.hub_type, hub.name, hub.addr
    );

    let hub = ConnectedHub::setup_hub(
        pu.create_hub(&hub).await.expect("Error creating hub"),
    )
    .await
    .expect("Error setting up hub");
    tokio::time::sleep(Duration::from_secs(1)).await; //Wait for attached devices to be collected

    // Devices to be used
    let hub_led: IoDevice;
    let motor_a: IoDevice;
    let motor_b: IoDevice;
    let motor_c: IoDevice;
    let motor_d: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hub_led = lock.io_from_kind(IoTypeId::HubLed)?;
        motor_a = lock.io_from_port(consts::named_port::A)?;
        motor_b = lock.io_from_port(consts::named_port::B)?;
        motor_c = lock.io_from_port(consts::named_port::C)?;
        motor_d = lock.io_from_port(consts::named_port::D)?;
    }

    // Set the hub LED if available
    println!("Setting hub LED");
    let colour = [0x00, 0xff, 0x00];
    println!("Setting to: {:02x?}", colour);
    hub_led.set_hubled_rgb(&colour).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    for motor in &[motor_a, motor_b, motor_c, motor_d] {
        motor.start_speed(50, 50).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        motor.start_power(Power::Float).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Done!");

    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("Disconnecting...");
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("Done");

    Ok(())
}
