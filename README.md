# Rust communication library for Lego Powered Up

[![](https://img.shields.io/github/license/sciguy16/lego-powered-up?style=plastic)](https://choosealicense.com/licenses/mpl-2.0/)
![](https://img.shields.io/github/workflow/status/sciguy16/lego-powered-up/build?style=plastic)
[![](https://img.shields.io/crates/v/lego-powered-up?style=plastic)](https://crates.io/crates/lego-powered-up)
[![](https://img.shields.io/docsrs/lego-powered-up?style=plastic)](https://docs.rs/lego-powered-up)


## Example

See the [examples](https://github.com/sciguy16/lego-powered-up/tree/main/examples) directory for more!

```rust
use lego_powered_up::{notifications::Power, PoweredUp};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Listening for hubs...");
    let mut pu = PoweredUp::init().await?;
    let hub = pu.wait_for_hub().await?;

    println!("Connecting to hub `{}`", hub.name);
    let hub = pu.create_hub(&hub).await?;

    println!("Change the hub LED to green");
    let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed).await?;
    hub_led.set_rgb(&[0, 0xff, 0]).await?;

    println!("Run motors");
    let mut motor_c = hub.port(lego_powered_up::hubs::Port::C).await?;
    let mut motor_d = hub.port(lego_powered_up::hubs::Port::D).await?;
    motor_c.start_speed(50, Power::Cw(50)).await?;
    motor_d.start_speed(50, Power::Cw(50)).await?;

    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Stop motors");
    motor_c.start_speed(0, Power::Float).await?;
    motor_d.start_speed(0, Power::Brake).await?;

    println!("Disconnect from hub `{}`", hub.name().await?);
    hub.disconnect().await?;

    println!("Done!");

    Ok(())
}
```

## Contributing
Contributions are welcome, particularly in the following areas:
* Bug reports and feature requests
* Support for hubs other than the Technic Medium Hub (I don't have any other types to test with at the moment)
* Support for peripherals other than the simple motors and hub LEDs
* Good APIs to control e.g. motor position
* More examples to demonstrate cool things we can do
* Client implementation
* `#![no_std]` support (controller & client)
* Testing on/porting to non-linux operating systems, e.g. Windows & Mac

## License
This library is available under the terms of the [Mozilla Public License 2.0](https://choosealicense.com/licenses/mpl-2.0/).

The examples provided in the [examples](https://github.com/sciguy16/lego-powered-up/tree/main/examples) directory are dedicated to the [public domain](https://creativecommons.org/publicdomain/zero/1.0/)
