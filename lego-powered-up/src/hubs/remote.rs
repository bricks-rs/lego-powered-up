use core::pin::Pin;
use crate::futures::stream::{Stream, StreamExt};
use crate::btleplug::api::ValueNotification;

use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::{notifications::*, NotificationHandler};
use crate::notifications::NetworkCommand::ConnectionRequest;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;
type HandlerMutex = Arc<Mutex<Box<dyn NotificationHandler>>>;

use crate::devices::remote::RcButtonState;
use crate::hubs::io_event::ChannelNotification;

#[derive(Debug, Copy, Clone)]
pub struct RemoteStatus {
    // Buttons
    pub a_plus: bool,
    pub a_red: bool,
    pub a_minus: bool,
    pub green: bool,
    pub b_plus: bool,
    pub b_red: bool,
    pub b_minus: bool,
    // Operaional status
    pub battery: u8,   // 0 - 100 %
    pub rssi: i8,      // -127 - 0  
}

impl RemoteStatus {
    pub fn new() -> Self {
        Self {
            a_plus: false,
            a_red: false,
            a_minus: false,
            green: false,
            b_plus: false,
            b_red: false,
            b_minus: false,
            battery: 100,
            rssi: 0
        }

    }
}



pub async fn rc_handler(
    mut stream: PinnedStream, 
    mutex: HubMutex, 
    hub_name: String,
    tx: broadcast::Sender::<RcButtonState>) {
        use crate::notifications::*;
        use crate::notifications::NetworkCommand::ConnectionRequest;
        while let Some(data) = stream.next().await {
            let r = NotificationMessage::parse(&data.value);
            match r {
                Ok(n) => {
                    match n {
                        NotificationMessage::HwNetworkCommands(cmd) => {
                            match cmd {
                                ConnectionRequest(state) => {
                                    match state {
                                        ButtonState::Up => { tx.send(RcButtonState::Green); }
                                        ButtonState::Released => { tx.send(RcButtonState::GreenUp); }
                                        _ => ()
                                    }    
                                }
                                _ => ()
                            }
                        }
                        NotificationMessage::PortValueSingle(val) => {
                            match val.port_id {
                                0x0 => {
                                    match val.data[0] {
                                        0 => { tx.send(RcButtonState::Aup); }
                                        1 => { tx.send(RcButtonState::Aplus); }
                                        127 => { tx.send(RcButtonState::Ared); }
                                        -1 => { tx.send(RcButtonState::Aminus); }
                                        _  => ()
                                    }
                                }
                                0x1 => {
                                    match val.data[0] {
                                        0 => { tx.send(RcButtonState::Bup); }
                                        1 => { tx.send(RcButtonState::Bplus); }
                                        127 => { tx.send(RcButtonState::Bred); }
                                        -1 => { tx.send(RcButtonState::Bminus); }
                                        _  => ()
                                    }
                                }
                                _ => ()                                
                            }
                        }
                    _ => ()
                    }
                }
                    Err(e) => {
                        println!("Parse error: {}", e);
                }
            }
        }  
}

// This one doesn't need to call NotificationMessage:parse for every BT notification.
// It does however by itself process PortValueSingles sent to every port...
// Also loses the green button which is communicated thru NotificationMessage::HwNetworkCommands
pub async fn rc_handler2( 
    mutex: HubMutex, 
    hub_name: String,
    tx: broadcast::Sender::<RcButtonState>,
    mut rx: broadcast::Receiver::<ChannelNotification>) {
        use crate::notifications::*;
        use crate::notifications::NetworkCommand::ConnectionRequest;
        while let Ok(data) = rx.recv().await {
            let d = data.portvaluesingle.unwrap(); 
            match d.port_id {
                0x0 => {
                    match d.data[0] {
                        0 => { tx.send(RcButtonState::Aup); }
                        1 => { tx.send(RcButtonState::Aplus); }
                        127 => { tx.send(RcButtonState::Ared); }
                        -1 => { tx.send(RcButtonState::Aminus); }
                        _  => ()
                    }
                }
                0x1 => {
                    match d.data[0] {
                        0 => { tx.send(RcButtonState::Bup); }
                        1 => { tx.send(RcButtonState::Bplus); }
                        127 => { tx.send(RcButtonState::Bred); }
                        -1 => { tx.send(RcButtonState::Bminus); }
                        _  => ()
                        }
                }
                _ => ()        
            }
                        // NotificationMessage::HwNetworkCommands(cmd) => {
                        //     match cmd {
                        //         ConnectionRequest(state) => {
                        //             match state {
                        //                 ButtonState::Up => { tx.send(RcButtonState::Green); }
                        //                 ButtonState::Released => { tx.send(RcButtonState::GreenUp); }
                        //                 _ => ()
                        //             }    
                        //         }
                        //         _ => ()
                        //     }
                        // }
        }
}  