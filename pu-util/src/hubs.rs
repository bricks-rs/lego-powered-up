use crate::argparse::HubArgs;
use anyhow::Result;
use lego_powered_up::PoweredUp;

pub fn run(args: &HubArgs) -> Result<()> {
    let mut pu = PoweredUp::init().unwrap();
    let rx = pu.event_receiver().unwrap();
    pu.start_scan().unwrap();

    while let Ok(evt) = rx.recv() {
        println!("Received event: {:?}", evt);
    }

    Ok(())
}
