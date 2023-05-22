// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::Result;
use argparse::Command;
use env_logger::Env;

mod argparse;
mod adapters;
mod hubs;
mod motor_test;

#[tokio::main]
async fn main() -> Result<()> {
    let args = argparse::parse_args();
    println!("verbosity: {}", args.verbosity);
    env_logger::Builder::from_env(Env::default().default_filter_or(
        match args.verbosity {
            0 => "warn",
            1 => "debug",
            _ => "trace",
        },
    ))
    .init();

    match args.command {
        Command::Devices(dev_args) => adapters::run(&dev_args).await?,
        Command::Hubs(hub_args) => hubs::run(&hub_args).await?,
        Command::MotorTest(mot_args) => motor_test::run(&mot_args).await?,
    }

    Ok(())
}
