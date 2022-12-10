// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use gilrs::{Button, Event, EventType::AxisChanged, Gilrs};
use lego_powered_up::devices::Device;
use lego_powered_up::hubs::Hub;
use lego_powered_up::notifications::Power;
use lego_powered_up::PoweredUp;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

struct Robot {
    speed_proportion: f32,
    steering_proportion: f32,
    left_speed: i8,
    right_speed: i8,
    left_motor: Box<dyn Device>,
    right_motor: Box<dyn Device>,
    changed: bool,
}

impl Display for Robot {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:3} {:3}", self.left_speed, self.right_speed)
    }
}

impl Robot {
    pub async fn set_steering(&mut self, steering: f32) {
        self.steering_proportion = steering.clamp(-1.0, 1.0);
        self.update();
    }

    pub async fn set_speed(&mut self, speed: f32) {
        self.speed_proportion = speed.clamp(-1.0, 1.0);
        self.update();
    }

    pub async fn new(
        hub: &dyn Hub,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let left_motor = hub.port(lego_powered_up::hubs::Port::C).await?;
        let right_motor = hub.port(lego_powered_up::hubs::Port::D).await?;
        Ok(Self {
            speed_proportion: 0.0,
            steering_proportion: 0.0,
            left_speed: 0,
            right_speed: 0,
            left_motor,
            right_motor,
            changed: false,
        })
    }

    // speed proportion is scaled to range [-80, 80]
    // steering proportion is scaled to range [-20, 20] and then
    // added/subtracted
    pub async fn send(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.left_motor
            .start_speed(self.left_speed, Power::from_i8(self.left_speed)?)
            .await?;
        self.right_motor
            .start_speed(self.right_speed, Power::from_i8(self.right_speed)?)
            .await?;
        Ok(())
    }

    pub fn changed(&mut self) -> bool {
        let c = self.changed;
        self.changed = false;
        c
    }

    fn update(&mut self) {
        let base_speed = (self.speed_proportion * 80.0).round() as i8;
        let mut steering_factor =
            (self.steering_proportion * 20.0).round() as i8;

        // if driving in reverse then invert the steering factor to make
        // the steering direction more natural
        if base_speed < 0 {
            steering_factor *= -1;
        }

        // if driving speed is close to 0 then boost the steering factor
        // a bit to make spinning in place more fun
        if base_speed.abs() < 20 {
            steering_factor *= 3;
        }

        let left_speed = base_speed + steering_factor;
        let right_speed = -(base_speed - steering_factor);

        self.changed = (self.left_speed != left_speed)
            || (self.right_speed != right_speed);
        self.left_speed = left_speed;
        self.right_speed = right_speed;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    println!("Searching for hubs...");
    let mut pu = PoweredUp::init().await?;
    let hub = pu.wait_for_hub().await?;

    println!("Connecting to hub `{}`", hub.name);
    let hub = pu.create_hub(&hub).await?;

    println!("Change the hub LED to green");
    let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed).await?;
    hub_led.set_rgb(&[0, 0xff, 0]).await?;

    let mut robot = Robot::new(hub.as_ref()).await?;

    let mut last_update = Instant::now();
    let min_time_between_updates = Duration::from_millis(100);

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            #[cfg(debug_assertions)]
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
            if let AxisChanged(axis, pos, _) = event {
                match axis {
                    gilrs::Axis::LeftStickX => robot.set_steering(pos).await,
                    gilrs::Axis::LeftStickY => robot.set_speed(pos).await,
                    gilrs::Axis::LeftZ => (),
                    gilrs::Axis::RightStickX => (),
                    gilrs::Axis::RightStickY => (),
                    gilrs::Axis::RightZ => (),
                    gilrs::Axis::DPadX => (),
                    gilrs::Axis::DPadY => (),
                    gilrs::Axis::Unknown => (),
                }
            }
        }

        if last_update.elapsed() > min_time_between_updates {
            last_update = Instant::now();
            if robot.changed() {
                println!("{}", robot);
                robot.send().await?;
            }
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
                break;
            }
        }
    }

    hub.disconnect().await?;

    Ok(())
}
