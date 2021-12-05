// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
    pub connect: bool,
}

pub struct MotorTestArgs {
    pub device_index: Option<usize>,
    pub address: String,
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
                        .about("Device index (from `devices`)")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .about("Search for hub with this name")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("address")
                        .long("address")
                        .about("Search for hub with this address")
                        .takes_value(true),
                )
                .arg(Arg::new("connect").long("connect").about(
                    "Connect to the discovered hub(s) and display more info",
                )),
        )
        .subcommand(
            App::new("motor-test")
                .about("Test motors connected to a hub")
                .arg(
                    Arg::new("device")
                        .long("device")
                        .about("Device index (from `devices`)")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("address")
                        .long("address")
                        .about("Address of hub")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .get_matches();

    let verbosity = min(matches.occurrences_of("verbose"), 2);

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
            connect: matches.is_present("connect"),
        })
    } else if let Some(matches) = matches.subcommand_matches("motor-test") {
        Command::MotorTest(MotorTestArgs {
            device_index: matches.value_of("device").map(|v| {
                v.parse()
                    .expect("Device index must be a nonnegative integer")
            }),
            address: matches.value_of("address").unwrap().to_string(),
        })
    } else {
        unreachable!();
    };

    Args { verbosity, command }
}
