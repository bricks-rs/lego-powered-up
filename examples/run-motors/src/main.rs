use lego_powered_up::notifications::Power;
use lego_powered_up::PoweredUp;
use std::{thread::sleep, time::Duration};

fn main() -> anyhow::Result<()> {
    println!("Listening for hubs...");
    let pu = PoweredUp::init()?;
    let hub = pu.wait_for_hub()?;

    println!("Connecting to hub `{}`", hub.name);
    let hub = pu.create_hub(&hub)?;

    println!("Change the hub LED to green");
    let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed)?;
    hub_led.set_rgb(&[0, 0xff, 0])?;

    println!("Run motors");
    let mut motor_c = hub.port(lego_powered_up::hubs::Port::C)?;
    let mut motor_d = hub.port(lego_powered_up::hubs::Port::D)?;
    motor_c.start_speed(50, Power::Cw(50))?;
    motor_d.start_speed(50, Power::Cw(50))?;

    sleep(Duration::from_secs(3));

    println!("Stop motors");
    motor_c.start_speed(0, Power::Float)?;
    motor_d.start_speed(0, Power::Brake)?;

    println!("Disconnect from hub `{}`", hub.get_name());
    hub.disconnect()?;

    println!("Done!");

    Ok(())
}
