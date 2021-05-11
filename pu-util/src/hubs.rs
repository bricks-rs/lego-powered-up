use crate::argparse::HubArgs;
use anyhow::Result;
use lego_powered_up::{Peripheral, PoweredUp, PoweredUpEvent};
use log::info;

pub fn run(args: &HubArgs) -> Result<()> {
    let mut pu = if let Some(dev) = args.device_index {
        PoweredUp::with_device(dev)?
    } else {
        PoweredUp::init()?
    };
    let rx = pu.event_receiver().unwrap();
    pu.start_scan().unwrap();

    println!("Listening for hub announcements...");

    while let Ok(evt) = rx.recv() {
        info!("Received event: {:?}", evt);
        if let PoweredUpEvent::HubDiscovered(hub_type, addr) = evt {
            let name = if let Some(peripheral) = pu.peripheral(addr) {
                peripheral.properties().local_name.unwrap_or("Unknown".to_string())
            } else {
                "Unknown".to_string()
            };
            println!(
                "Discovered `{}` `{}` with address `{}`",
                hub_type, name, addr
            );
        }
    }

    Ok(())
}
