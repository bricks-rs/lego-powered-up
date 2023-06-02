// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

// #![allow(unused)]
use lego_powered_up::{PoweredUp, ConnectedHub, IoDevice}; 
use lego_powered_up::consts::named_port;
use lego_powered_up::devices::remote::RcDevice;
use lego_powered_up::devices::remote::RcButtonState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init PoweredUp with found adapter
    println!("Looking for BT adapter and initializing PoweredUp with found adapter");
    let mut pu = PoweredUp::init().await?;

    println!("Waiting for hubs...");
    let hub= pu.wait_for_hub().await?;

    println!("Connecting to hub...");
    let rc_hub = ConnectedHub::setup_hub
                                        (pu.create_hub(&hub).await.expect("Error creating hub"))
                                        .await.expect("Error setting up hub");

    // Set up RC input 
    let rc: IoDevice;
    {
        let lock = rc_hub.mutex.lock().await;
        rc = lock.io_from_port(named_port::A).await?;
    }    
    let (mut rc_rx, _) = rc.remote_connect_with_green().await?;

    // Print some feedback for button presses. Both red buttons together to exit.
    let button_feedback = tokio::spawn(async move {
        let mut red_down: (bool, bool) = (false, false); 
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
