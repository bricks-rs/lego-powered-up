/// Convenience functions

use crate::{PoweredUp, ConnectedHub};
use crate::{HubFilter, }; 
use crate::error::{Result, }; 

// use crate::{IoDevice, IoTypeId}; 
// use crate::consts::named_port;
// use crate::iodevice::modes;
// use crate::iodevice::remote::{RcDevice, RcButtonState};
// use crate::iodevice::{light::*, sensor::*, motor::*};

/// Setup single hub
pub async fn single_hub() -> Result<ConnectedHub> {
    println!("Discovering BT adapter and initializing PoweredUp");
    let mut pu = PoweredUp::init().await?;
    println!("Waiting for hub...");
    let hub= pu.wait_for_hub().await?;
    println!("Connecting to hub...");
     
    Ok(ConnectedHub::setup_hub(pu.create_hub(&hub)
        .await.expect("Error creating hub"))
        .await.expect("Error setting up hub")
    )
}

/// Setup main hub + remote control
pub async fn main_and_rc() -> Result<(ConnectedHub, ConnectedHub)> {
        println!("Discovering BT adapter and initializing PoweredUp");
        let mut pu = PoweredUp::init().await?;
        let hub_count = 2;
        println!("Waiting for hubs...");
        let discovered_hubs = pu.wait_for_hubs_filter(HubFilter::Null, &hub_count).await?;
        println!("Discovered {} hubs, trying to connect...", &hub_count);
    
        let mut connected_hubs: Vec<ConnectedHub> = Vec::new();
        for dh in discovered_hubs {
            println!("Connecting to hub `{}`", dh.name);
            let created_hub = pu.create_hub(&dh).await?;
            connected_hubs.push(ConnectedHub::setup_hub(created_hub).await.expect("Error setting up hub `{}`, dh.name"))
        }
    
        let rc_hub: ConnectedHub;
        let main_hub: ConnectedHub;
        match connected_hubs[0].kind {
            crate::consts::HubType::RemoteControl => {
                rc_hub = connected_hubs.remove(0);
                main_hub = connected_hubs.remove(0); 
            }
            _ => {
                main_hub = connected_hubs.remove(0);
                rc_hub = connected_hubs.remove(0); 
            }
        }
        Ok( (main_hub, rc_hub) )    
    
}


