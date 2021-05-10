use lego_powered_up::PoweredUp;

fn main() {
    env_logger::init();
    println!("Hello, world!");

    let mut pu = PoweredUp::init().unwrap();
    let rx = pu.event_receiver().unwrap();
    pu.start_scan().unwrap();

    while let Ok(evt) = rx.recv() {
        println!("Received event: {:?}", evt);
    }
}
