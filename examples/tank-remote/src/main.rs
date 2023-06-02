// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use lego_powered_up::{PoweredUp, ConnectedHub, IoTypeId, IoDevice,
    consts,
    devices::light::{self, HubLed},
    devices::motor::{EncoderMotor, Power},
    Result as LpuResult 
};
use console_engine::{pixel, Color, ConsoleEngine, KeyCode};
use eyre::Result;
use std::fmt::{self, Display, Formatter};

struct Robot {
    left_speed: i8,
    right_speed: i8,
    left_motor: IoDevice,
    right_motor: IoDevice,
}

impl Display for Robot {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:3} ~ {:3}", self.left_speed, self.right_speed)
    }
}

impl Robot {
    pub fn new(
        left_motor: IoDevice,
        right_motor: IoDevice,
    ) -> Self {
        Self {
            left_speed: 0,
            right_speed: 0,
            left_motor,
            right_motor,
        }
    }

    pub async fn stop(&mut self) -> LpuResult<()> {
        self.left_speed = 0;
        self.right_speed = 0;
        self.commit().await
    }

    pub async fn forward(&mut self) -> LpuResult<()> {
        self.left_speed = 50;
        self.right_speed = 50;
        self.commit().await
    }

    pub async fn backward(&mut self) -> LpuResult<()> {
        self.left_speed = -50;
        self.right_speed = -50;
        self.commit().await
    }

    pub async fn left(&mut self) -> LpuResult<()> {
        self.left_speed = -50;
        self.right_speed = 50;
        self.commit().await
    }

    pub async fn right(&mut self) -> LpuResult<()> {
        self.left_speed = 50;
        self.right_speed = -50;
        self.commit().await
    }

    async fn commit(&mut self) -> LpuResult<()> {
        self.left_motor
            .start_speed(self.left_speed, Power::from_i8(self.left_speed)?)
            .await?;
        self.right_motor
            .start_speed(self.right_speed, Power::from_i8(self.right_speed)?)
            .await?;
        Ok(())
    }
}

fn key(engine: &mut ConsoleEngine, key: KeyCode) -> bool {
    engine.is_key_pressed(key) || engine.is_key_held(key)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Listening for hubs...");
    let mut pu = PoweredUp::init().await?;
    let hub = pu.wait_for_hub().await?;

    println!("Connecting to hub `{}`", hub.name);
    let hub = ConnectedHub::setup_hub
                                        (pu.create_hub(&hub).await.expect("Error creating hub"))
                                        .await.expect("Error setting up hub");

    // Devices to be used
    let hub_led: IoDevice;
    let motor_a: IoDevice;
    let motor_b: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hub_led = lock.io_from_kind(IoTypeId::HubLed).await?;
        motor_a = lock.io_from_port(consts::named_port::A).await?; 
        motor_b = lock.io_from_port(consts::named_port::B).await?; 
    }    
    
    println!("Change the hub LED to green");
    hub_led.set_hubled_mode(light::HubLedMode::Colour).await?;
    hub_led.set_hubled_color(consts::Color::Green).await?;

    // initializes a screen of 20x10 characters with a target of 3 frames
    // per second
    // coordinates will range from [0,0] to [19,9]
    let mut engine = console_engine::ConsoleEngine::init(20, 20, 5)?;

    let mut robot = Robot::new(motor_a, motor_b);

    loop {
        //TODO ascii art robot
        engine.wait_frame(); // wait for next frame + capture inputs
        engine.clear_screen(); // reset the screen
                               // draw a line of '#' from [0,0] to [19,9]
        engine.line(0, 0, 19, 0, pixel::pxl('#'));
        // prints some value at [0,4]
        engine.print(0, 4, format!("Robot: {}", robot).as_str());

        // write a majestic cyan 'O' at [4,0]
        engine.set_pxl(4, 0, pixel::pxl_fg('O', Color::Cyan));

        if key(&mut engine, KeyCode::Up) {
            robot.forward().await?;
        } else if key(&mut engine, KeyCode::Down) {
            robot.backward().await?;
        } else if key(&mut engine, KeyCode::Left) {
            robot.left().await?;
        } else if key(&mut engine, KeyCode::Right) {
            robot.right().await?;
        } else {
            robot.stop().await?;
        }
        if engine.is_key_pressed(KeyCode::Char('q')) {
            // if the user presses 'q' :
            break;
        }

        engine.draw(); // draw the screen
    }

    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("Exit successful");

    Ok(())
}
