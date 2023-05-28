use core::pin::Pin;
use crate::futures::stream::{Stream, StreamExt};
use crate::btleplug::api::ValueNotification;

use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::broadcast;

type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;


#[derive(Debug, Copy, Clone)]
pub enum RcButtonState {
    Aup,
    Aplus,
    Ared,
    Aminus,
    Bup,
    Bplus,
    Bred,
    Bminus,
    Green,
    GreenUp
}

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
                            match val.values[0] {
                                0x0 => {
                                    match val.values[1] {
                                        0 => { tx.send(RcButtonState::Aup); }
                                        1 => { tx.send(RcButtonState::Aplus); }
                                        127 => { tx.send(RcButtonState::Ared); }
                                        255 => { tx.send(RcButtonState::Aminus); }
                                        _  => ()
                                    }
                                }
                                0x1 => {
                                    match val.values[1] {
                                        0 => { tx.send(RcButtonState::Bup); }
                                        1 => { tx.send(RcButtonState::Bplus); }
                                        127 => { tx.send(RcButtonState::Bred); }
                                        255 => { tx.send(RcButtonState::Bminus); }
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