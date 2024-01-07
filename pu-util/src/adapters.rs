// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::argparse::DevicesArgs;
use anyhow::Result;
use lego_powered_up::{btleplug::api::Central, PoweredUp};

pub async fn run(args: &DevicesArgs) -> Result<()> {
    let adapters = PoweredUp::adapters().await?;

    if let Some(idx) = args.index {
        if let Some(adapter) = adapters.get(idx) {
            println!("Showing 1 Bluetooth adapter:");
            println!("  {}: {}", idx, adapter.adapter_info().await?);
        } else {
            println!("No Bluetooth adapter found");
        }
        return Ok(());
    }

    if adapters.is_empty() {
        println!("No Bluetooth adapter found");
    } else {
        println!("Showing {} available Bluetooth adapters:", adapters.len());
        for (idx, dev) in adapters.iter().enumerate() {
            println!("  {}: {}", idx, dev.adapter_info().await?);
        }
    }
    Ok(())
}
