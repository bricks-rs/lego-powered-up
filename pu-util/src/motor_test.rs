// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::argparse::MotorTestArgs;
use anyhow::Result;
use lego_powered_up::notifications::Power;
use lego_powered_up::{consts::HubType, hubs::Port, DiscoveredHub, PoweredUp};

use std::thread::sleep;
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
    let hub = pu.wait_for_hub().await?;

    println!(
        "Connecting to `{}` `{}` with address `{}`",
        hub.hub_type, hub.name, hub.addr
    );

    let hub = pu.create_hub(&hub).await?;

    println!("Setting hub LED");

    // Set the hub LED if available
    let mut hub_led = hub.port(Port::HubLed).await?;
    let colour = [0x00, 0xff, 0x00];
    println!("Setting to: {:02x?}", colour);
    hub_led.set_rgb(&colour)?;
    sleep(Duration::from_secs(1));

    for port in &[Port::A, Port::B, Port::C, Port::D] {
        let mut motor = hub.port(*port).await?;
        motor.start_speed(50, Power::Cw(100))?;
        sleep(Duration::from_secs(2));
        motor.start_speed(0, Power::Float)?;
        sleep(Duration::from_secs(1));
    }

    println!("Done!");

    sleep(Duration::from_secs(5));

    println!("Disconnecting...");
    hub.disconnect().await?;
    println!("Done");

    pu.stop().await?;

    Ok(())
}
