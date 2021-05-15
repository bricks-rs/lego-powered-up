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
        let hub = pu.create_hub(hub).await?;

        println!("Setting hub LED");

        // Set the hub LED if available
        let mut hub_led = hub.port(Port::HubLed).await?;
        for colour in [[0, 0xff, 0], [0xff, 0, 0], [0, 0, 0xff]]
            .iter()
            .cycle()
            .take(10)
        {
            println!("Setting to: {:?}", colour);
            hub_led.set_colour(&colour).await?;
            sleep(Duration::from_secs(1));
        }

        println!("Done!");

        sleep(Duration::from_secs(5));

        println!("Disconnecting...");
        hub.disconnect().await?;
        println!("Done");
    }

    Ok(())
}
