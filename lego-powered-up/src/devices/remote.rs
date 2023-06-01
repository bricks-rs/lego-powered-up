use async_trait::async_trait;
use core::fmt::Debug;
use crate::{Error, Result};
use crate::notifications::NotificationMessage;
use crate::notifications::InputSetupSingle;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use core::pin::Pin;
use crate::futures::stream::{Stream, StreamExt};
use crate::btleplug::api::ValueNotification;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
type HubMutex = Arc<Mutex<Box<dyn crate::Hub>>>;
type PinnedStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;
use crate::{notifications::*};
use crate::notifications::NetworkCommand::ConnectionRequest;

struct MsgWrapper {
    pvs_msg: Option<PortValueSingleFormat>,
    nwc_msg: Option<NetworkCommand>
}
impl MsgWrapper {
    pub fn pvs(msg: PortValueSingleFormat) -> Self{
        Self {
            pvs_msg: Some(msg),
            nwc_msg: None
        }
    }
    pub fn nwc(msg: NetworkCommand) -> Self{
        Self {
            nwc_msg: Some(msg),
            pvs_msg: None
        }
    }
}

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

#[async_trait]
pub trait RcDevice: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;
    fn check(&self) -> Result<()>;
    fn get_rx_pvs(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn get_rx_nwc(&self) -> Result<broadcast::Receiver<NetworkCommand>>;

    async fn remote_buttons_enable(&self, mode: u8, delta: u32) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        let p = match self.p() {
            Some(p) => p,
            None => return Err(Error::NoneError((String::from("Not a rc button device"))))
        };
        crate::hubs::send(p, self.c().unwrap(), mode_set_msg).await
    }

    async fn remote_buttons_enable_both(&self) -> Result<()> {
        let p = self.p().expect("Peripheral not in device cache");
        let c = self.c().expect("Charactheristic not in device cache");
        for port in 0..2 { // For some reason the last for-loop is dropped?
            // println!("Enable buttons on port: {:?}", port);
            let mode_set_msg =
                NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                    port_id: port,
                    mode: 0,                // Not sure what the usecases for the different button modes are, 0 seems fine
                    delta: 1,
                    notification_enabled: true,
                });
           
            crate::hubs::send2(&p, &c, mode_set_msg).await.expect("Error while setting mode");
        }

        Ok(())
    }

    async fn remote_connect(&self) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()> )> {
        match self.check() {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not a remote control device"))))
        }
        self.remote_buttons_enable_both().await?;

        // Set up channel
        let (tx, mut rx) = broadcast::channel::<RcButtonState>(8);
        let mut pvs_from_main = self.get_rx_pvs().expect("Single value sender not in device cache");
        let nwc_from_main = self.get_rx_nwc().expect("Network command sender not in device cache");
                let task = tokio::spawn(async move {
                    while let Ok(msg) = pvs_from_main.recv().await {
                        match msg.port_id {
                            0x0 => {
                                match msg.data[0] as i8 {
                                    0 => { tx.send(RcButtonState::Aup); }
                                    1 => { tx.send(RcButtonState::Aplus); }
                                    127 => { tx.send(RcButtonState::Ared); }
                                    -1 => { tx.send(RcButtonState::Aminus); }
                                    _  => ()
                                }
                            }
                            0x1 => {
                                match msg.data[0] as i8 {
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
                });
            
                Ok((rx, task))
      
    }

    

    async fn remote_connect_with_green(&self) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()> )> {
        match self.check() {
            Ok(()) => (),
            _ => return Err(Error::NoneError((String::from("Not a remote control device"))))
        }
        self.remote_buttons_enable_both().await?;

        // Set up channel
        let (tx, mut rx) = broadcast::channel::<RcButtonState>(8);
        let mut pvs_from_main = self.get_rx_pvs().expect("Single value sender not in device cache");
        let mut nwc_from_main = self.get_rx_nwc().expect("Network command sender not in device cache");
                let task = tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            Ok(msg) = pvs_from_main.recv() => {
                                match msg.port_id {
                                    0x0 => {
                                        match msg.data[0] as i8 {
                                            0 => { tx.send(RcButtonState::Aup); }
                                            1 => { tx.send(RcButtonState::Aplus); }
                                            127 => { tx.send(RcButtonState::Ared); }
                                            -1 => { tx.send(RcButtonState::Aminus); }
                                            _  => ()
                                        }
                                    }
                                    0x1 => {
                                        match msg.data[0] as i8 {
                                            0 => { tx.send(RcButtonState::Bup); }
                                            1 => { tx.send(RcButtonState::Bplus); }
                                            127 => { tx.send(RcButtonState::Bred); }
                                            -1 => { tx.send(RcButtonState::Bminus); }
                                            _  => ()
                                        }
                                    }
                                    _ => ()                                
                                } 
                            },
                            Ok(msg) = nwc_from_main.recv() => {
                                match msg {
                                    NetworkCommand::ConnectionRequest(ButtonState::Up) => { tx.send(RcButtonState::Green); },
                                    NetworkCommand::ConnectionRequest(ButtonState::Released) => { tx.send(RcButtonState::GreenUp); },
                                    _ => ()
                                }    
                            },
                            else => { break }
                        };    
                    }
                });
            
                Ok((rx, task))
    }

}



// #[derive(Default )]
pub struct RcHandler {
    stream: PinnedStream,
    // hub_mutex: HubMutex,
    // hub_name: String,
    self_mutex: Option<Arc<Mutex<Box<RcHandler>>>>,
    tx: mpsc::Sender<RcButtonState>,
    rx: Option<mpsc::Receiver<RcButtonState>>
}

// impl NotificationHandler for RcHandler {
// }

// impl RcHandler {
//     pub fn new(stream: PinnedStream,
//                 tx: mpsc::Sender<RcButtonState> 
//                 // hub_mutex: HubMutex, 
//                 // hub_name: String
//             ) -> Self {
//         Self {
//             stream,
//             // hub_mutex,
//             // hub_name,
//             self_mutex: None,
//             tx,
//             rx: None
//         }
//     }

//     pub async fn process(&mut self) -> () {
//             while let Some(data) = self.stream.next().await {
//                 println!("RcHandler received data from {:?} [{:?}]: {:?}", "hub name", data.uuid, data.value);
//                 let r = NotificationMessage::parse(&data.value);
//                 match r {
//                     Ok(n) => {
//                         match n {
//                             NotificationMessage::HwNetworkCommands(cmd) => {
//                                 match cmd {
//                                     ConnectionRequest(state) => {
//                                         match state {
//                                             ButtonState::Up => { self.tx.send(RcButtonState::Green); }
//                                             ButtonState::Released => { self.tx.send(RcButtonState::GreenUp); }
//                                             _ => ()
//                                         }    
//                                     }
//                                     _ => ()
//                                 }
//                             }
//                             NotificationMessage::PortValueSingle(val) => {
//                                 match val.port_id {
//                                     0x0 => {
//                                         match val.data[0] {
//                                             0 => { self.tx.send(RcButtonState::Aup); }
//                                             1 => { self.tx.send(RcButtonState::Aplus); }
//                                             127 => { self.tx.send(RcButtonState::Ared); }
//                                             -1 => { self.tx.send(RcButtonState::Aminus); }
//                                             _  => ()
//                                         }
//                                     }
//                                     0x1 => {
//                                         match val.data[0] {
//                                             0 => { self.tx.send(RcButtonState::Bup); }
//                                             1 => { self.tx.send(RcButtonState::Bplus); }
//                                             127 => { self.tx.send(RcButtonState::Bred); }
//                                             -1 => { self.tx.send(RcButtonState::Bminus); }
//                                             _  => ()
//                                         }
//                                     }
//                                     _ => ()                                
//                                 }
//                             }
//                         _ => ()
//                         }
//                     }
//                         Err(e) => {
//                             println!("Parse error: {}", e);
//                     }
//                 }
//             }  
//     }
// }


// type HandlerMutex = Arc<Mutex<Box<dyn NotificationHandler>>>;
// use crate::hubs::io_event::ChannelNotification;

// #[derive(Debug, Copy, Clone)]
// pub struct RemoteStatus {
//     // Buttons
//     pub a_plus: bool,
//     pub a_red: bool,
//     pub a_minus: bool,
//     pub green: bool,
//     pub b_plus: bool,
//     pub b_red: bool,
//     pub b_minus: bool,
//     // Operaional status
//     pub battery: u8,   // 0 - 100 %
//     pub rssi: i8,      // -127 - 0  
// }

// impl RemoteStatus {
//     pub fn new() -> Self {
//         Self {
//             a_plus: false,
//             a_red: false,
//             a_minus: false,
//             green: false,
//             b_plus: false,
//             b_red: false,
//             b_minus: false,
//             battery: 100,
//             rssi: 0
//         }

//     }
// }

// pub async fn rc_handler(
//     mut stream: PinnedStream, 
//     mutex: HubMutex, 
//     hub_name: String,
//     tx: broadcast::Sender::<RcButtonState>) {

//         while let Some(data) = stream.next().await {
//             let r = NotificationMessage::parse(&data.value);
//             match r {
//                 Ok(n) => {
//                     match n {
//                         NotificationMessage::HwNetworkCommands(cmd) => {
//                             match cmd {
//                                 ConnectionRequest(state) => {
//                                     match state {
//                                         ButtonState::Up => { tx.send(RcButtonState::Green); }
//                                         ButtonState::Released => { tx.send(RcButtonState::GreenUp); }
//                                         _ => ()
//                                     }    
//                                 }
//                                 _ => ()
//                             }
//                         }
//                         NotificationMessage::PortValueSingle(val) => {
//                             match val.port_id {
//                                 0x0 => {
//                                     match val.data[0] {
//                                         0 => { tx.send(RcButtonState::Aup); }
//                                         1 => { tx.send(RcButtonState::Aplus); }
//                                         127 => { tx.send(RcButtonState::Ared); }
//                                         -1 => { tx.send(RcButtonState::Aminus); }
//                                         _  => ()
//                                     }
//                                 }
//                                 0x1 => {
//                                     match val.data[0] {
//                                         0 => { tx.send(RcButtonState::Bup); }
//                                         1 => { tx.send(RcButtonState::Bplus); }
//                                         127 => { tx.send(RcButtonState::Bred); }
//                                         -1 => { tx.send(RcButtonState::Bminus); }
//                                         _  => ()
//                                     }
//                                 }
//                                 _ => ()                                
//                             }
//                         }
//                     _ => ()
//                     }
//                 }
//                     Err(e) => {
//                         println!("Parse error: {}", e);
//                 }
//             }
//         }  
// }

// This one doesn't need to call NotificationMessage:parse for every BT notification.
// It does however by itself process PortValueSingles sent to every port...
// Also loses the green button which is communicated thru NotificationMessage::HwNetworkCommands
// pub async fn rc_handler2( 
//     mutex: HubMutex, 
//     hub_name: String,
//     tx: broadcast::Sender::<RcButtonState>,
//     mut rx: broadcast::Receiver::<ChannelNotification>) {
//         while let Ok(data) = rx.recv().await {
//             let d = data.portvaluesingle.unwrap(); 
//             match d.port_id {
//                 0x0 => {
//                     match d.data[0] {
//                         0 => { tx.send(RcButtonState::Aup); }
//                         1 => { tx.send(RcButtonState::Aplus); }
//                         127 => { tx.send(RcButtonState::Ared); }
//                         -1 => { tx.send(RcButtonState::Aminus); }
//                         _  => ()
//                     }
//                 }
//                 0x1 => {
//                     match d.data[0] {
//                         0 => { tx.send(RcButtonState::Bup); }
//                         1 => { tx.send(RcButtonState::Bplus); }
//                         127 => { tx.send(RcButtonState::Bred); }
//                         -1 => { tx.send(RcButtonState::Bminus); }
//                         _  => ()
//                         }
//                 }
//                 _ => ()        
//             }
//                         // NotificationMessage::HwNetworkCommands(cmd) => {
//                         //     match cmd {
//                         //         ConnectionRequest(state) => {
//                         //             match state {
//                         //                 ButtonState::Up => { tx.send(RcButtonState::Green); }
//                         //                 ButtonState::Released => { tx.send(RcButtonState::GreenUp); }
//                         //                 _ => ()
//                         //             }    
//                         //         }
//                         //         _ => ()
//                         //     }
//                         // }
//         }
// }  