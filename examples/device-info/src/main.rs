// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

// #![allow(unused)]
use core::time::Duration;

use std::sync::{Arc};
use tokio::time::sleep as sleep;
use tokio::sync::Mutex;

use lego_powered_up::{IoDevice, IoTypeId};
use lego_powered_up::{PoweredUp, Hub, ConnectedHub,}; 
use lego_powered_up::consts::{LEGO_COLORS};
use lego_powered_up::devices::{light::*};

type HubMutex = Arc<Mutex<Box<dyn Hub>>>;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Discovering BT adapter and initializing PoweredUp");
    let mut pu = PoweredUp::init().await?;

    println!("Waiting for hub...");
    let hub = pu.wait_for_hub().await?;
    
    println!("Connecting to hub...");
    let hub = ConnectedHub::setup_hub(pu.create_hub(&hub)
                                        .await.expect("Error creating hub"))
                                        .await.expect("Error setting up hub");

    // Demo hub RGB 
    let hubled: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hubled = lock.io_from_kind(IoTypeId::HubLed).await?;
    }
    tokio::spawn(async move { 
        // LEGO colors
        hubled.set_hubled_mode(HubLedMode::Colour).await.expect("Error setting mode");
        for c in LEGO_COLORS {
                hubled.set_hubled_color(c).await.expect("Error setting color");
                sleep(Duration::from_millis(500)).await;
        }
        sleep(Duration::from_millis(1000)).await;

        // Rainbow
        hubled.set_hubled_mode(HubLedMode::Rgb).await.expect("Error setting mode");
        let mut rgb: [u8; 3] = [0; 3];
        loop {
            for angle in 0..360 {
                rgb[0] = RAINBOW_TABLE[(angle+120)%360];
                rgb[1] = RAINBOW_TABLE[angle];
                rgb[2] = RAINBOW_TABLE[(angle+240)%360];
                hubled.set_hubled_rgb(&rgb).await.expect("Error setting RGB");    
                sleep(Duration::from_millis(30)).await;
            }
        }
    });

    // Start attached io ui
    let mutex = hub.mutex.clone();
    ui(mutex).await;

    // Cleanup after ui exit
    let lock = hub.mutex.lock().await;
    println!("Disconnect from hub `{}`", lock.name().await?);
    lock.disconnect().await?;
    println!("Done!");

    Ok(())
}


pub async fn ui(mutex: HubMutex) -> () {
    use text_io::read;
    loop {
        print!("(l)ist, <port> or (q)uit > ");
        let line: String = read!("{}\n");
        if line.len() == 1 {
            continue;
        } 
        else if line.contains("l") {
            let mut lock = mutex.lock().await;
            for device in lock.connected_io().values() {
                println!("{}", device);
            }
            continue;
        } 
        else if line.contains("q") {
            break
        }
        else {
            let input = line.trim().parse::<u8>();
            match input {
                Ok(num) => {
                    let mut lock = mutex.lock().await;
                    let o = lock.connected_io().get(&num);
                    match o {
                        Some(device) => { println!("{:#?}", device) }  //{dbg!(device);}
                        None => {println!("Device not found");}
                    }
                }
                Err(e) => {
                    println!("Not a number: {}", e);
                }
            }
        }
    }
}


const RAINBOW_TABLE: [u8; 360] = [
    0,   0,   0,   0,   0,   1,   1,   2, 
    2,   3,   4,   5,   6,   7,   8,   9, 
   11,  12,  13,  15,  17,  18,  20,  22, 
   24,  26,  28,  30,  32,  35,  37,  39, 
   42,  44,  47,  49,  52,  55,  58,  60, 
   63,  66,  69,  72,  75,  78,  81,  85, 
   88,  91,  94,  97, 101, 104, 107, 111, 
  114, 117, 121, 124, 127, 131, 134, 137, 
  141, 144, 147, 150, 154, 157, 160, 163, 
  167, 170, 173, 176, 179, 182, 185, 188, 
  191, 194, 197, 200, 202, 205, 208, 210, 
  213, 215, 217, 220, 222, 224, 226, 229, 
  231, 232, 234, 236, 238, 239, 241, 242, 
  244, 245, 246, 248, 249, 250, 251, 251, 
  252, 253, 253, 254, 254, 255, 255, 255, 
  255, 255, 255, 255, 254, 254, 253, 253, 
  252, 251, 251, 250, 249, 248, 246, 245, 
  244, 242, 241, 239, 238, 236, 234, 232, 
  231, 229, 226, 224, 222, 220, 217, 215, 
  213, 210, 208, 205, 202, 200, 197, 194, 
  191, 188, 185, 182, 179, 176, 173, 170, 
  167, 163, 160, 157, 154, 150, 147, 144, 
  141, 137, 134, 131, 127, 124, 121, 117, 
  114, 111, 107, 104, 101,  97,  94,  91, 
   88,  85,  81,  78,  75,  72,  69,  66, 
   63,  60,  58,  55,  52,  49,  47,  44, 
   42,  39,  37,  35,  32,  30,  28,  26, 
   24,  22,  20,  18,  17,  15,  13,  12, 
   11,   9,   8,   7,   6,   5,   4,   3, 
    2,   2,   1,   1,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0, 
    0,   0,   0,   0,   0,   0,   0,   0
];