// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::argparse::HubArgs;
use anyhow::Result;
use lego_powered_up::HubFilter;
use lego_powered_up::{
    consts,
    iodevice::hubled::{self, HubLed},
    iodevice::motor::{EncoderMotor, Power},
    ConnectedHub, IoDevice, IoTypeId, PoweredUp,
};
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
        let hub = ConnectedHub::setup_hub(
            pu.create_hub(&hub).await.expect("Error creating hub"),
        )
        .await
        .expect("Error setting up hub");
        tokio::time::sleep(Duration::from_secs(1)).await; //Wait for attached devices to be collected

        // Set the hub LED if available
        println!("Setting hub LED");
        let hub_led: IoDevice;
        {
            let lock = hub.mutex.lock().await;
            hub_led = lock.io_from_kind(IoTypeId::HubLed)?;
        }
        hub_led.set_hubled_mode(hubled::HubLedMode::Colour)?;
        for colour in [[0_u8, 0xff, 0], [0xff, 0, 0], [0, 0, 0xff]]
            .iter()
            .cycle()
            .take(10)
        {
            println!("Setting to: {:02x?}", colour);
            hub_led.set_hubled_rgb(colour)?;
            tokio::time::sleep(Duration::from_millis(400)).await;
        }

        println!("Setting Motor A");
        let motor: IoDevice;
        {
            let lock = hub.mutex.lock().await;
            motor = lock.io_from_port(consts::named_port::A)?;
        }
        motor.start_speed(50, 50)?;
        tokio::time::sleep(Duration::from_secs(4)).await;
        motor.start_power(Power::Float)?;

        println!("Done!");

        tokio::time::sleep(Duration::from_secs(2)).await;

        println!("Disconnecting...");
        {
            let lock = hub.mutex.lock().await;
            lock.disconnect().await?;
        }
        println!("Done");
    }

    Ok(())
}
