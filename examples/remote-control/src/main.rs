// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

// #![allow(unused)]
use lego_powered_up::IoDevice; 
use lego_powered_up::consts::named_port;
use lego_powered_up::iodevice::remote::{RcDevice, RcButtonState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rc_hub = lego_powered_up::setup::single_hub().await?;

    // Set up RC input 
    let rc: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc = lock.io_from_port(named_port::A).await?;
    }    
    let (mut rc_rx, _rc_task) = rc.remote_connect_with_green().await?;

    // Print some feedback for button presses. Both red buttons together to exit.
    let button_feedback = tokio::spawn(async move {
        let mut red_down = (false, false); 
        while let Ok(data) = rc_rx.recv().await {
            match data {
                RcButtonState::Aup => { 
                    println!("A released");
                    red_down.0 = false;
                }
                RcButtonState::Aplus => { println!("A plus") }
                RcButtonState::Ared => { 
                    println!("A red"); 
                    red_down.0 = true; 
                }
                RcButtonState::Aminus => { println!("A minus") }
                RcButtonState::Bup => { 
                    println!("B released");
                    red_down.1 = false; 
                }
                RcButtonState::Bplus => { println!("B plus") }
                RcButtonState::Bred => { 
                    println!("B red");
                    red_down.1 = true;
                }
                RcButtonState::Bminus => { println!("B minus") }
                RcButtonState::Green => { println!("Green pressed") }
                RcButtonState::GreenUp => { println!("Green released") }
            }
            if red_down == (true, true) { break }
        }
    });
    button_feedback.await?;

    // Cleanup 
    println!("Disconnect from hub `{}`", rc_hub.name);
    {
        let lock = rc_hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("Done!");

    Ok(())
}
