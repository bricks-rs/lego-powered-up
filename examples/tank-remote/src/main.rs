// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use console_engine::{pixel, Color, ConsoleEngine, KeyCode};
use eyre::Result;
use lego_powered_up::notifications::Power;
use lego_powered_up::{PortController, PoweredUp, Result as LpuResult};
use std::fmt::{self, Display, Formatter};

struct Robot {
    left_speed: i8,
    right_speed: i8,
    left_motor: PortController,
    right_motor: PortController,
}

impl Display for Robot {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:3} ~ {:3}", self.left_speed, self.right_speed)
    }
}

impl Robot {
    pub fn new(
        left_motor: PortController,
        right_motor: PortController,
    ) -> Self {
        Self {
            left_speed: 0,
            right_speed: 0,
            left_motor,
            right_motor,
        }
    }

    pub fn stop(&mut self) -> LpuResult<()> {
        self.left_speed = 0;
        self.right_speed = 0;
        self.commit()
    }

    pub fn forward(&mut self) -> LpuResult<()> {
        self.left_speed = 50;
        self.right_speed = -50;
        self.commit()
    }

    pub fn backward(&mut self) -> LpuResult<()> {
        self.left_speed = -50;
        self.right_speed = 50;
        self.commit()
    }

    pub fn left(&mut self) -> LpuResult<()> {
        self.left_speed = -50;
        self.right_speed = -50;
        self.commit()
    }

    pub fn right(&mut self) -> LpuResult<()> {
        self.left_speed = 50;
        self.right_speed = 50;
        self.commit()
    }

    fn commit(&mut self) -> LpuResult<()> {
        self.left_motor
            .start_speed(self.left_speed, Power::from_i8(self.left_speed)?)?;
        self.right_motor
            .start_speed(self.right_speed, Power::from_i8(self.right_speed)?)?;
        Ok(())
    }
}

fn key(engine: &mut ConsoleEngine, key: KeyCode) -> bool {
    engine.is_key_pressed(key) || engine.is_key_held(key)
}

fn main() -> Result<()> {
    println!("Searching for hubs...");
    let pu = PoweredUp::init()?;
    let hub = pu.wait_for_hub()?;

    println!("Connecting to hub `{}`", hub.name);
    let hub = pu.create_hub(&hub)?;

    println!("Change the hub LED to green");
    let mut hub_led = hub.port(lego_powered_up::hubs::Port::HubLed)?;
    hub_led.set_rgb(&[0, 0xff, 0])?;

    let motor_c = hub.port(lego_powered_up::hubs::Port::C)?;
    let motor_d = hub.port(lego_powered_up::hubs::Port::D)?;

    // initializes a screen of 20x10 characters with a target of 3 frames per second
    // coordinates will range from [0,0] to [19,9]
    let mut engine = console_engine::ConsoleEngine::init(20, 20, 5)?;

    let mut robot = Robot::new(motor_c, motor_d);

    loop {
        //TODO ascii art robot
        engine.wait_frame(); // wait for next frame + capture inputs
        engine.clear_screen(); // reset the screen

        engine.line(0, 0, 19, 0, pixel::pxl('#')); // draw a line of '#' from [0,0] to [19,9]
        engine.print(0, 4, format!("Robot: {}", robot).as_str()); // prints some value at [0,4]

        engine.set_pxl(4, 0, pixel::pxl_fg('O', Color::Cyan)); // write a majestic cyan 'O' at [4,0]

        if key(&mut engine, KeyCode::Up) {
            robot.forward()?;
        } else if key(&mut engine, KeyCode::Down) {
            robot.backward()?;
        } else if key(&mut engine, KeyCode::Left) {
            robot.left()?;
        } else if key(&mut engine, KeyCode::Right) {
            robot.right()?;
        } else {
            robot.stop()?;
        }
        if engine.is_key_pressed(KeyCode::Char('q')) {
            // if the user presses 'q' :
            break;
        }

        engine.draw(); // draw the screen
    }

    hub.disconnect()?;

    println!("Exit successful");

    Ok(())
}
