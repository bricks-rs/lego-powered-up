use crate::notifications::Power;
use crate::{hubs::Port, HubManagerMessage, NotificationMessage};
use anyhow::{bail, Result};
use async_trait::async_trait;
use btleplug::api::BDAddr;
use std::fmt::Debug;
use tokio::sync::{mpsc::Sender, oneshot};

#[async_trait]
pub trait Device: Debug + Send + Sync {
    fn port(&self) -> Port;
    async fn send(&mut self, _msg: NotificationMessage) -> Result<()>;
    async fn set_rgb(&mut self, _rgb: &[u8; 3]) -> Result<()> {
        bail!("Not implemented for type")
    }
    async fn start_speed(
        &mut self,
        _speed: i8,
        _max_power: Power,
    ) -> Result<()> {
        bail!("Not implemented for type")
    }
}

pub(crate) fn create_device(
    port_id: u8,
    port_type: Port,
    hub_addr: BDAddr,
    hub_manager_tx: Sender<HubManagerMessage>,
) -> Box<dyn Device + Send + Sync> {
    match port_type {
        Port::HubLed => {
            let dev = HubLED::new(hub_addr, hub_manager_tx, port_id);
            Box::new(dev)
        }
        Port::A | Port::B | Port::C | Port::D => {
            let dev = Motor::new(hub_addr, hub_manager_tx, port_type, port_id);
            Box::new(dev)
        }
        _ => todo!(),
    }
}

#[derive(Debug, Clone)]
pub struct HubLED {
    rgb: [u8; 3],
    mode: HubLedMode,
    port_id: u8,
    hub_addr: BDAddr,
    hub_manager_tx: Sender<HubManagerMessage>,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HubLedMode {
    Colour = 0x0,
    Rgb = 0x01,
}

#[async_trait]
impl Device for HubLED {
    fn port(&self) -> Port {
        Port::HubLed
    }

    async fn send(&mut self, msg: NotificationMessage) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();
        self.hub_manager_tx
            .send(HubManagerMessage::SendToHub(self.hub_addr, msg, tx))
            .await?;
        rx.await?
    }

    async fn set_rgb(&mut self, rgb: &[u8; 3]) -> Result<()> {
        use crate::notifications::*;

        self.rgb = *rgb;

        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: 50,
                mode: 0x01,
                delta: 0x00000001,
                notification_enabled: false,
            });
        self.send(mode_set_msg).await?;

        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColors {
                red: rgb[0],
                green: rgb[1],
                blue: rgb[2],
            },
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }
}

impl HubLED {
    pub(crate) fn new(
        hub_addr: BDAddr,
        hub_manager_tx: Sender<HubManagerMessage>,
        port_id: u8,
    ) -> Self {
        let mode = HubLedMode::Rgb;
        //hub.subscribe(mode);
        Self {
            rgb: [0; 3],
            mode,
            port_id,
            hub_addr,
            hub_manager_tx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Motor {
    port: Port,
    port_id: u8,
    hub_addr: BDAddr,
    hub_manager_tx: Sender<HubManagerMessage>,
}

#[async_trait]
impl Device for Motor {
    fn port(&self) -> Port {
        self.port
    }

    async fn send(&mut self, msg: NotificationMessage) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<()>>();
        self.hub_manager_tx
            .send(HubManagerMessage::SendToHub(self.hub_addr, msg, tx))
            .await?;
        rx.await?
    }

    async fn start_speed(&mut self, speed: i8, max_power: Power) -> Result<()> {
        use crate::notifications::*;

        let subcommand = PortOutputSubcommand::StartSpeed {
            speed,
            max_power,
            use_acc_profile: true,
            use_dec_profile: true,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: self.port_id,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        self.send(msg).await
    }
}

impl Motor {
    pub(crate) fn new(
        hub_addr: BDAddr,
        hub_manager_tx: Sender<HubManagerMessage>,
        port: Port,
        port_id: u8,
    ) -> Self {
        Self {
            port,
            port_id,
            hub_addr,
            hub_manager_tx,
        }
    }
}
