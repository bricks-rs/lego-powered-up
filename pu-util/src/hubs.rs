use crate::argparse::HubArgs;
use anyhow::Result;
use lego_powered_up::{
    consts::HubType, hubs::Port, BDAddr, DiscoveredHub, HubFilter, PoweredUp,
};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

pub fn run(args: &HubArgs) -> Result<()> {
    let mut pu = if let Some(dev) = args.device_index {
        PoweredUp::with_device(dev)?
    } else {
        PoweredUp::init()?
    };
    // let rx = pu.event_receiver().unwrap();
    // pu.run().unwrap();

    println!("Listening for hub announcements...");

    // 90:84:2B:60:3C:B8
    // 90:84:2B:60:3A:6C

    let hub = if let Some(addr) = &args.address {
        DiscoveredHub {
            hub_type: HubType::Unknown,
            addr: BDAddr::from_str(addr)?,
            name: "".to_string(),
        }
    } else if let Some(name) = &args.name {
        pu.wait_for_hub_filter(HubFilter::Name(name.to_string()))?
    } else {
        pu.wait_for_hub()?
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
        let hub = pu.create_hub(&hub)?;

        println!("Setting hub LED");

        // Set the hub LED if available
        let mut hub_led = hub.port(Port::HubLed)?;
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

        let mut motor = hub.port(Port::A)?;
        motor.start_speed(50, Power::Cw(50))?;
        sleep(Duration::from_secs(4));
        motor.start_speed(0, Power::Float)?;

        println!("Done!");

        sleep(Duration::from_secs(5));

        println!("Disconnecting...");
        hub.disconnect()?;
        println!("Done");
    }
    pu.stop()?;

    Ok(())
}
