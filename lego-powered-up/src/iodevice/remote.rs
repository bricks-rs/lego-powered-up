use crate::Result;
/// Support for the button devices in
/// https://rebrickable.com/parts/28739/control-unit-powered-up/
/// The unit as a whole functions as a hub that connects the
/// two button devices, hubled and voltage and rssi sensors.
use async_trait::async_trait;
use core::fmt::Debug;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::device_trait;
use crate::hubs::Tokens;
use crate::notifications::{
    ButtonState, InputSetupSingle,
    NetworkCommand::{self},
    NotificationMessage, PortValueSingleFormat,
};

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
    GreenUp,
}

device_trait!(RcDevice, [
    fn get_rx_pvs(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;,
    fn get_rx_nwc(&self) -> Result<broadcast::Receiver<NetworkCommand>>;,

    async fn remote_buttons_enable(&self, mode: u8, delta: u32) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode,
                delta,
                notification_enabled: true,
            });
        self.commit(msg).await
    },

    async fn remote_buttons_enable_by_port(&self, port_id: u8) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id,
                mode: 0, // Not sure what the usecases for the different button modes are, 0 seems fine
                delta: 1,
                notification_enabled: true,
            });
        self.commit(msg).await
    },

    async fn remote_connect(
        &self,
    ) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()>)> {
        self.remote_buttons_enable_by_port(0x0).await?;
        self.remote_buttons_enable_by_port(0x1).await?;

        // Set up channel
        let (tx, rx) = broadcast::channel::<RcButtonState>(64);
        let mut pvs_from_main = self
            .get_rx_pvs()
            .expect("Single value sender not in device cache");
        let task = tokio::spawn(async move {
            while let Ok(msg) = pvs_from_main.recv().await {
                match msg.port_id {
                    0x0 => match msg.data[0] {
                        0 => {
                            let _ = tx.send(RcButtonState::Aup);
                        }
                        1 => {
                            let _ = tx.send(RcButtonState::Aplus);
                        }
                        127 => {
                            let _ = tx.send(RcButtonState::Ared);
                        }
                        -1 => {
                            let _ = tx.send(RcButtonState::Aminus);
                        }
                        _ => (),
                    },
                    0x1 => match msg.data[0] {
                        0 => {
                            let _ = tx.send(RcButtonState::Bup);
                        }
                        1 => {
                            let _ = tx.send(RcButtonState::Bplus);
                        }
                        127 => {
                            let _ = tx.send(RcButtonState::Bred);
                        }
                        -1 => {
                            let _ = tx.send(RcButtonState::Bminus);
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
        });

        Ok((rx, task))
    },

    async fn remote_connect_with_green(
        &self,
    ) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()>)> {
        self.remote_buttons_enable_by_port(0x0).await?;
        self.remote_buttons_enable_by_port(0x1).await?;

        // Set up channel
        let (tx, rx) = broadcast::channel::<RcButtonState>(8);
        let mut pvs_from_main = self
            .get_rx_pvs()
            .expect("Single value sender not in device cache");
        let mut nwc_from_main = self
            .get_rx_nwc()
            .expect("Network command sender not in device cache");
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(msg) = pvs_from_main.recv() => {
                        match msg.port_id {
                            0x0 => {
                                match msg.data[0] {
                                    0 => { let _ = tx.send(RcButtonState::Aup); }
                                    1 => { let _ = tx.send(RcButtonState::Aplus); }
                                    127 => { let _ = tx.send(RcButtonState::Ared); }
                                    -1 => { let _ = tx.send(RcButtonState::Aminus); }
                                    _  => ()
                                }
                            }
                            0x1 => {
                                match msg.data[0] {
                                    0 => { let _ = tx.send(RcButtonState::Bup); }
                                    1 => { let _ = tx.send(RcButtonState::Bplus); }
                                    127 => { let _ = tx.send(RcButtonState::Bred); }
                                    -1 => { let _ = tx.send(RcButtonState::Bminus); }
                                    _  => ()
                                }
                            }
                            _ => ()
                        }
                    },
                    Ok(msg) = nwc_from_main.recv() => {
                        match msg {
                            NetworkCommand::ConnectionRequest(ButtonState::Up) => { let _ = tx.send(RcButtonState::Green); },
                            NetworkCommand::ConnectionRequest(ButtonState::Released) => { let _ = tx.send(RcButtonState::GreenUp); },
                            _ => ()
                        }
                    },
                    else => { break }
                };
            }
        });

        Ok((rx, task))
    }
]);

/*
#[async_trait]
pub trait RcDevice: Debug + Send + Sync {
    /// Device trait boilerplate
    fn port(&self) -> u8;
    fn get_rx_pvs(&self) -> Result<broadcast::Receiver<PortValueSingleFormat>>;
    fn get_rx_nwc(&self) -> Result<broadcast::Receiver<NetworkCommand>>;
    fn check(&self) -> Result<()>;
    fn tokens(&self) -> Tokens;
    async fn commit(&self, msg: NotificationMessage) -> Result<()> {
        match crate::hubs::send(self.tokens(), msg).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn remote_buttons_enable(&self, mode: u8, delta: u32) -> Result<()> {
        self.check()?;
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode,
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
                mode: 0, // Not sure what the usecases for the different button modes are, 0 seems fine
                delta: 1,
                notification_enabled: true,
            });
        self.commit(msg).await
    }

    async fn remote_connect(
        &self,
    ) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()>)> {
        self.remote_buttons_enable_by_port(0x0).await?;
        self.remote_buttons_enable_by_port(0x1).await?;

        // Set up channel
        let (tx, rx) = broadcast::channel::<RcButtonState>(64);
        let mut pvs_from_main = self
            .get_rx_pvs()
            .expect("Single value sender not in device cache");
        let task = tokio::spawn(async move {
            while let Ok(msg) = pvs_from_main.recv().await {
                match msg.port_id {
                    0x0 => match msg.data[0] {
                        0 => {
                            let _ = tx.send(RcButtonState::Aup);
                        }
                        1 => {
                            let _ = tx.send(RcButtonState::Aplus);
                        }
                        127 => {
                            let _ = tx.send(RcButtonState::Ared);
                        }
                        -1 => {
                            let _ = tx.send(RcButtonState::Aminus);
                        }
                        _ => (),
                    },
                    0x1 => match msg.data[0] {
                        0 => {
                            let _ = tx.send(RcButtonState::Bup);
                        }
                        1 => {
                            let _ = tx.send(RcButtonState::Bplus);
                        }
                        127 => {
                            let _ = tx.send(RcButtonState::Bred);
                        }
                        -1 => {
                            let _ = tx.send(RcButtonState::Bminus);
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
        });

        Ok((rx, task))
    }

    async fn remote_connect_with_green(
        &self,
    ) -> Result<(broadcast::Receiver<RcButtonState>, JoinHandle<()>)> {
        self.remote_buttons_enable_by_port(0x0).await?;
        self.remote_buttons_enable_by_port(0x1).await?;

        // Set up channel
        let (tx, rx) = broadcast::channel::<RcButtonState>(8);
        let mut pvs_from_main = self
            .get_rx_pvs()
            .expect("Single value sender not in device cache");
        let mut nwc_from_main = self
            .get_rx_nwc()
            .expect("Network command sender not in device cache");
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(msg) = pvs_from_main.recv() => {
                        match msg.port_id {
                            0x0 => {
                                match msg.data[0] {
                                    0 => { let _ = tx.send(RcButtonState::Aup); }
                                    1 => { let _ = tx.send(RcButtonState::Aplus); }
                                    127 => { let _ = tx.send(RcButtonState::Ared); }
                                    -1 => { let _ = tx.send(RcButtonState::Aminus); }
                                    _  => ()
                                }
                            }
                            0x1 => {
                                match msg.data[0] {
                                    0 => { let _ = tx.send(RcButtonState::Bup); }
                                    1 => { let _ = tx.send(RcButtonState::Bplus); }
                                    127 => { let _ = tx.send(RcButtonState::Bred); }
                                    -1 => { let _ = tx.send(RcButtonState::Bminus); }
                                    _  => ()
                                }
                            }
                            _ => ()
                        }
                    },
                    Ok(msg) = nwc_from_main.recv() => {
                        match msg {
                            NetworkCommand::ConnectionRequest(ButtonState::Up) => { let _ = tx.send(RcButtonState::Green); },
                            NetworkCommand::ConnectionRequest(ButtonState::Released) => { let _ = tx.send(RcButtonState::GreenUp); },
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
 */