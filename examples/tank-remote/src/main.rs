use console_engine::{pixel, Color, ConsoleEngine, KeyCode};
use eyre::Result;
use std::fmt::{self, Display, Formatter};

#[derive(Default)]
struct Robot {
    left_speed: i8,
    right_speed: i8,
}

impl Display for Robot {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{:3} ~ {:3}", self.left_speed, self.right_speed)
    }
}

impl Robot {
    pub fn stop(&mut self) {
        self.left_speed = 0;
        self.right_speed = 0;
    }

    pub fn forward(&mut self) {
        self.left_speed = 50;
        self.right_speed = 50;
    }

    pub fn backward(&mut self) {
        self.left_speed = -50;
        self.right_speed = -50;
    }
}

fn key(engine: &mut ConsoleEngine, key: KeyCode) -> bool {
    engine.is_key_pressed(key) || engine.is_key_held(key)
}

fn main() -> Result<()> {
    println!("Searching for hubs...");

    // initializes a screen of 20x10 characters with a target of 3 frames per second
    // coordinates will range from [0,0] to [19,9]
    let mut engine = console_engine::ConsoleEngine::init(20, 20, 5)?;

    let mut robot = Robot::default();

    loop {
        engine.wait_frame(); // wait for next frame + capture inputs
        engine.clear_screen(); // reset the screen

        engine.line(0, 0, 19, 0, pixel::pxl('#')); // draw a line of '#' from [0,0] to [19,9]
        engine.print(0, 4, format!("Robot: {robot}").as_str()); // prints some value at [0,4]

        engine.set_pxl(4, 0, pixel::pxl_fg('O', Color::Cyan)); // write a majestic cyan 'O' at [4,0]

        if key(&mut engine, KeyCode::Up) {
            robot.forward();
        } else if key(&mut engine, KeyCode::Down) {
            robot.backward();
        } else {
            robot.stop();
        }
        if engine.is_key_pressed(KeyCode::Char('q')) {
            // if the user presses 'q' :
            break; // exits app
        }

        engine.draw(); // draw the screen
    }

    println!("Exit successful");

    Ok(())
}
