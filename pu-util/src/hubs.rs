use crate::argparse::HubArgs;
use anyhow::{Context, Result};
use lego_powered_up::{
    devices::HubLED, hubs::Port, HubFilter, Peripheral, PoweredUp,
};
use log::info;
use std::thread::sleep;
use std::time::Duration;

pub async fn run(args: &HubArgs) -> Result<()> {
    let pu = if let Some(dev) = args.device_index {
        PoweredUp::with_device(dev)?
    } else {
        PoweredUp::init()?
    };
    // let rx = pu.event_receiver().unwrap();
    // pu.run().unwrap();

    println!("Listening for hub announcements...");

    let hub = if let Some(addr) = &args.address {
        pu.wait_for_hub_filter(HubFilter::Addr(addr.to_string()))
            .await?
    } else if let Some(name) = &args.name {
        pu.wait_for_hub_filter(HubFilter::Name(name.to_string()))
            .await?
    } else {
        pu.wait_for_hub().await?
    };

    let hub_type = hub.hub_type;
    let name = hub.name;

    let verb = if args.connect {
        "Connecting to"
    } else {
        "Discovered"
    };
    println!(
        "{} `{}` `{}` with address `{}`",
        verb, hub_type, name, hub.addr
    );

    if args.connect {
        let hub = pu.create_hub(hub_type, hub.addr)?;

        println!("Setting hub LED");

        // Set the hub LED if available
        if hub.port_map().contains_key(&Port::HubLed) {
            for colour in [[0, 0xff, 0], [0xff, 0, 0], [0, 0, 0xff]]
                .iter()
                .cycle()
                .take(10)
            {
                while let Some(_msg) = hub.poll() {
                    println!("[pu-util]: msg received");
                }
                println!("Setting to: {:?}", colour);
                let mut led = HubLED::new();
                led.set_colour(&colour, &hub)?;
                sleep(Duration::from_secs(1));
            }
        }

        println!("Done!");

        sleep(Duration::from_secs(5));

        println!("Disconnecting...");
        hub.disconnect()?;
        println!("Done");
    }

    Ok(())
}
