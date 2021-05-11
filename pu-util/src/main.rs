use anyhow::Result;
use argparse::Command;
use env_logger::Env;

mod argparse;
mod devices;
mod hubs;
mod motor_test;

fn main() -> Result<()> {
    let args = argparse::parse_args();
    env_logger::Builder::from_env(Env::default().default_filter_or(
        match args.verbosity {
            0 => "warn",
            1 => "debug",
            _ => "trace",
        },
    ))
    .init();

    match args.command {
        Command::Devices(dev_args) => devices::run(&dev_args)?,
        Command::Hubs(hub_args) => hubs::run(&hub_args)?,
        _ => todo!(),
    }

    Ok(())
}
