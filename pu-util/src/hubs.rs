// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::argparse::HubArgs;
use anyhow::Result;
use lego_powered_up::{hubs::Port, HubFilter, PoweredUp};

use std::thread::sleep;
use std::time::Duration;

pub async fn run(args: &HubArgs) -> Result<()> {
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

    let hub = if let Some(addr) = &args.address {
        pu.wait_for_hub_filter(HubFilter::Addr(addr.to_string()))
            .await?
    } else if let Some(name) = &args.name {
        pu.wait_for_hub_filter(HubFilter::Name(name.to_string()))
            .await?
    } else {
        pu.wait_for_hub().await?
    };

    let verb = if args.connect {
        "Connecting to"
    } else {
        "Discovered"
    };
    println!(
        "{} `{}` `{}` with address `{}`",
        verb, hub.hub_type, hub.name, hub.addr
    );

    if args.connect {
        use lego_powered_up::notifications::Power;
        let hub = pu.create_hub(&hub).await?;

        println!("Setting hub LED");

        // Set the hub LED if available
        let mut hub_led = hub.port(Port::HubLed).await?;
        for colour in [[0_u8, 0xff, 0], [0xff, 0, 0], [0, 0, 0xff]]
            .iter()
            .cycle()
            .take(10)
        {
            println!("Setting to: {:02x?}", colour);
            hub_led.set_rgb(colour)?;
            sleep(Duration::from_secs(1));
        }

        println!("Setting Motor A");

        let mut motor = hub.port(Port::A).await?;
        motor.start_speed(50, Power::Cw(50))?;
        sleep(Duration::from_secs(4));
        motor.start_speed(0, Power::Float)?;

        println!("Done!");

        sleep(Duration::from_secs(5));

        println!("Disconnecting...");
        hub.disconnect().await?;
        println!("Done");
    }
    pu.stop().await?;

    Ok(())
}
