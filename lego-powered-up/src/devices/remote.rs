use async_trait::async_trait;
use core::fmt::Debug;
use crate::{Error, Result};
use btleplug::api::{Characteristic};
use btleplug::platform::Peripheral;
use tokio::sync::broadcast;
// use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::notifications::{ NetworkCommand::{self, ConnectionRequest}, PortValueSingleFormat, NotificationMessage,
                            InputSetupSingle, ButtonState};


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
    fn port(&self) -> u8;
    fn tokens(&self) -> (&Peripheral, &Characteristic);
    fn get_rx_pvs(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn get_rx_nwc(&self) -> Result<broadcast::Receiver<NetworkCommand>>;
    fn check(&self) -> Result<()>;

    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
        let tokens = self.tokens();
        match crate::hubs::send2(tokens.0, tokens.1, msg).await { 
            Ok(()) => Ok(()),
            Err(e)  => { Err(e) }
        }
    }

    async fn remote_buttons_enable(&self, mode: u8, delta: u32) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        self.commit(msg).await
    }

    async fn remote_buttons_enable_by_port(&self, port_id: u8) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id,
                mode: 0,                // Not sure what the usecases for the different button modes are, 0 seems fine
                delta: 1,
                notification_enabled: true,
            });
        self.commit(msg).await
    }

    async fn remote_connect(&self) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()> )> {
        self.remote_buttons_enable_by_port(0x0).await?;
        self.remote_buttons_enable_by_port(0x1).await?;

        // Set up channel
        let (tx, mut rx) = broadcast::channel::<RcButtonState>(8);
        let mut pvs_from_main = self.get_rx_pvs().expect("Single value sender not in device cache");
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
        self.remote_buttons_enable_by_port(0x0).await?;
        self.remote_buttons_enable_by_port(0x1).await?;

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

