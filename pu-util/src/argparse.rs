use clap::{crate_version, App, AppSettings, Arg};
use std::cmp::min;

pub struct Args {
    pub verbosity: u64,
    pub command: Command,
}

pub enum Command {
    Devices(DevicesArgs),
    Hubs(HubArgs),
    MotorTest(MotorTestArgs),
}

pub struct DevicesArgs {
    pub index: Option<usize>,
}

pub struct HubArgs {
    pub device_index: Option<usize>,
    pub address: Option<String>,
    pub name: Option<String>,
}

pub struct MotorTestArgs {
    pub hub: Option<String>,
}

pub fn parse_args() -> Args {
    let matches = App::new("PoweredUp Util")
        .version(crate_version!())
        .author("David Young https://github.com/sciguy16/lego-powered-up")
        .about("Discover and test Lego PoweredUp devices")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("verbose")
                .short('v')
                .multiple(true)
                .takes_value(false)
                .about("Increase verbosity"),
        )
        .subcommand(
            App::new("devices")
                .about("Information about connected Bluetooth devices")
                .arg(Arg::new("index")),
        )
        .subcommand(
            App::new("hubs")
                .about("Scan for PoweredUp hubs")
                .arg(
                    Arg::new("device")
                        .long("device")
                        .about("Device index (from `devices`)"),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .about("Search for hub with this name"),
                )
                .arg(
                    Arg::new("address")
                        .long("address")
                        .about("Search for hub with this address"),
                ),
        )
        .get_matches();

    let verbosity = min(matches.occurrences_of("v"), 2);

    let command = if let Some(matches) = matches.subcommand_matches("devices") {
        let index = matches
            .value_of("index")
            .map(|v| v.parse().expect("Index must be a nonnegative integer"));

        Command::Devices(DevicesArgs { index })
    } else if let Some(matches) = matches.subcommand_matches("hubs") {
        Command::Hubs(HubArgs {
            device_index: matches.value_of("device").map(|v| {
                v.parse()
                    .expect("Device index must be a nonnegative integer")
            }),
            name: matches.value_of("name").map(String::from),
            address: matches.value_of("address").map(String::from),
        })
    } else {
        unreachable!();
    };

    Args { verbosity, command }
}
