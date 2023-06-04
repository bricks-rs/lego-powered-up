// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::time::Duration;
use tokio::time::sleep as sleep;

use lego_powered_up::{setup, devices};
use lego_powered_up::{PoweredUp, ConnectedHub, IoDevice, IoTypeId}; 
use lego_powered_up::{Hub, HubFilter, }; 
use lego_powered_up::error::{Error, Result, OptionContext}; 
use lego_powered_up::consts::named_port;
use lego_powered_up::notifications::Power;
use lego_powered_up::consts::{LEGO_COLORS, };
use lego_powered_up::devices::modes;
use lego_powered_up::devices::remote::{RcDevice, RcButtonState};
use lego_powered_up::devices::{light::*, sensor::*, motor::*};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === Single hub === 
    let hub = setup::single_hub().await?;

    // Do stuff
    use lego_powered_up::devices::iodevice::definition::ModeKind;
    use lego_powered_up::devices::iodevice::definition::PortMode;
    let mut d_list: Vec<IoDevice> = Vec::new(); 
    {
        let mut lock = hub.mutex.lock().await;
        // let devices = lock.connected_io().values().filter(
        //         |&x| x.modes.values().kind == ModeKind::Sensor)
        //     .map(|x|x.clone()).collect();
        let temp: Vec<IoDevice> = lock.connected_io().values().map(|x|x.clone()).collect();
        for mut d in temp {
            // if d.def.modes().pop_first().unwrap().1.kind == ModeKind::Sensor {
            //     d_list.push(d);
            // }
        }
    }
    for mut d in d_list {
        let mode_count = &d.def.modes().len();
        let (mut d_rx, _) = d.raw_channel().await.unwrap();
        tokio::spawn(async move {
            // let m: Vec<&PortMode> = d.modes.values().collect();
            // d.set_device_mode(d.modes.pop_first().unwrap().0, 1).await;
            while let Ok(data) = d_rx.recv().await {
                println!("Rssi: {:?} {:?}", data, data[0] as i8)
            }
        });    
    }

    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    Ok(())
}

