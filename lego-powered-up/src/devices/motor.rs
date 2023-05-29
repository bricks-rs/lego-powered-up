use async_trait::async_trait;
use core::fmt::Debug;
use crate::{Error, Result};
use crate::notifications::NotificationMessage;
use crate::notifications::InputSetupSingle;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

use futures::stream::StreamExt;

use crate::PinnedStream;
use crate::HubMutex;


#[derive(Debug, Copy, Clone)]
pub enum MotorState{
    Speed(i8),
    Pos(i16),
    Apos(i16)
}

#[async_trait]
pub trait EncoderMotor: Debug + Send + Sync {
    fn p(&self) -> Option<Peripheral>;
    fn c(&self) -> Option<Characteristic>;
    fn port(&self) -> u8;

    async fn motor_sensor_enable(&self, mode: u8, delta: u32) -> Result<()> {
        let mode_set_msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: self.port(),
                mode: mode as u8,
                delta,
                notification_enabled: true,
            });
        let p = match self.p() {
            Some(p) => p,
            None => return Err(Error::NoneError((String::from("Not an Encoder Motor"))))
        };
        crate::hubs::send(p, self.c().unwrap(), mode_set_msg).await
    }
}


pub async fn motor_handler(mut stream: PinnedStream, mutex: HubMutex, hub_name: String) {
    while let Some(data) = stream.next().await {
        // println!("Received data from {:?} [{:?}]: {:?}", hub_name, data.uuid, data.value);

        let r = NotificationMessage::parse(&data.value);
        match r {
            Ok(n) => {
                // dbg!(&n);
                match n {
                    // Active feedback
                    NotificationMessage::PortValueSingle(val) => {

                    }
                    NotificationMessage::PortValueCombinedmode(val) => {}

                    // Setup feedback and errors
                    NotificationMessage::PortInputFormatSingle(val) => {}
                    NotificationMessage::PortInputFormatCombinedmode(val) => {}
                    NotificationMessage::PortOutputCommandFeedback(val ) => {}
                    NotificationMessage::GenericErrorMessages(val) => {}

                    _ => ()
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

    }  
}