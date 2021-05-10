use lego_powered_up::{PoweredUp};

fn main() {
    println!("Hello, world!");

    let mut pu = PoweredUp::init().unwrap();
    let rx = pu.event_receiver().unwrap();
    pu.start_scan();

    while let Ok(evt) = rx.recv() {

    }
}
