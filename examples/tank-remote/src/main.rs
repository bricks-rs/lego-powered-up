use crossterm::event::{self, Event, KeyCode};
use crossterm::style::{self, Stylize};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, QueueableCommand};
use eyre::Result;
use std::fmt::{self, Display, Formatter};
use std::io::{stdout, Write};
use std::time::Duration;

#[derive(Default)]
struct Robot {
    left_speed: i8,
    right_speed: i8,
}

impl Display for Robot {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{} - {}", self.left_speed, self.right_speed)
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
}

fn main() -> Result<()> {
    println!("Searching for hubs...");

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut robot = Robot::default();
    stdout.queue(cursor::MoveTo(5, 5))?;
    loop {
        match event::poll(Duration::from_millis(500))? {
            true => {
                if let Event::Key(key) = event::read()? {
                    use KeyCode::*;
                    //println!("{:?}", key);
                    match key.code {
                        Char('q') => break,
                        Up => robot.forward(),
                        _ => (),
                    }
                }

                stdout.flush()?;
            }
            false => {
                robot.stop();
            }
        }
        stdout
            .queue(cursor::MoveTo(5, 5))?
            .queue(style::PrintStyledContent(format!("{robot}").magenta()))?;
    }

    execute!(stdout, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    println!("Exit successful");

    Ok(())
}
