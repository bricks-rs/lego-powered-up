use super::*;

// Macros
use crate::ok;
use crate::next;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum NotificationMessage {
    HubProperties(HubProperty),
    HubActions(HubAction),
    HubAlerts(AlertType),
    HubAttachedIo(AttachedIo),
    GenericErrorMessages(ErrorMessageFormat),
    HwNetworkCommands(NetworkCommand),
    FwUpdateGoIntoBootMode([u8; 9]),
    FwUpdateLockMemory([u8; 8]),
    FwUpdateLockStatusRequest,
    FwLockStatus(LockStatus),
    PortInformationRequest(InformationRequest),
    PortModeInformationRequest(ModeInformationRequest),
    PortInputFormatSetupSingle(InputSetupSingle),
    PortInputFormatSetupCombinedmode(InputSetupCombined),
    PortInformation(PortInformationValue),
    PortModeInformation(PortModeInformationValue),
    PortValueSingle(PortValueSingleFormat),
    PortValueCombinedmode(PortValueCombinedFormat),
    PortInputFormatSingle(PortInputFormatSingleFormat),
    PortInputFormatCombinedmode(PortInputFormatCombinedFormat),
    VirtualPortSetup(VirtualPortSetupFormat),
    PortOutputCommand(PortOutputCommandFormat),
    PortOutputCommandFeedback(PortOutputCommandFeedbackFormat),
}

impl NotificationMessage {
    /// Parse a byte slice into a notification message
    pub fn parse(msg: &[u8]) -> Result<Self> {
        use NotificationMessage::*;

        debug!("NOTIFICATION: {:?}", msg);

        let mut msg_iter = msg.iter();

        // consume the length bytes
        Self::validate_length(&mut msg_iter, msg.len())?;
        trace!("Length: {}", msg.len());

        let _hub_id = next!(msg_iter);
        trace!("Hub ID: {}", _hub_id);

        let message_type = ok!(MessageType::from_u8(next!(msg_iter)));

        trace!(
            "Identified message type: {:?} = {:x}",
            message_type,
            message_type as u8
        );

        Ok(match message_type {
            MessageType::HubProperties => {
                let props = HubProperty::parse(&mut msg_iter)?;
                HubProperties(props)
            }
            MessageType::HubActions => {
                let action = HubAction::parse(&mut msg_iter)?;
                HubActions(action)
            }
            MessageType::HubAlerts => {
                let alert = AlertType::parse(&mut msg_iter)?;
                HubAlerts(alert)
            }
            MessageType::HubAttachedIo => {
                let attach = AttachedIo::parse(&mut msg_iter)?;
                HubAttachedIo(attach)
            }
            MessageType::GenericErrorMessages => {
                let error = ErrorMessageFormat::parse(&mut msg_iter)?;
                GenericErrorMessages(error)
            }
            MessageType::HwNetworkCommands => {
                let command = NetworkCommand::parse(&mut msg_iter)?;
                HwNetworkCommands(command)
            }
            MessageType::FwUpdateGoIntoBootMode => {
                let mut safety = [0_u8; 9];
                for ele in safety.iter_mut() {
                    *ele = next!(msg_iter);
                }
                FwUpdateGoIntoBootMode(safety)
            }
            MessageType::FwUpdateLockMemory => {
                let mut safety = [0_u8; 8];
                for ele in safety.iter_mut() {
                    *ele = next!(msg_iter);
                }
                FwUpdateLockMemory(safety)
            }
            MessageType::FwUpdateLockStatusRequest => FwUpdateLockStatusRequest,
            MessageType::FwLockStatus => {
                let status = LockStatus::parse(&mut msg_iter)?;
                FwLockStatus(status)
            }
            MessageType::PortInformationRequest => {
                let req = InformationRequest::parse(&mut msg_iter)?;
                PortInformationRequest(req)
            }
            MessageType::PortModeInformationRequest => {
                let req = ModeInformationRequest::parse(&mut msg_iter)?;
                PortModeInformationRequest(req)
            }
            MessageType::PortInputFormatSetupSingle => {
                let setup = InputSetupSingle::parse(&mut msg_iter)?;
                PortInputFormatSetupSingle(setup)
            }
            MessageType::PortInputFormatSetupCombinedmode => {
                let setup = InputSetupCombined::parse(&mut msg_iter)?;
                PortInputFormatSetupCombinedmode(setup)
            }
            MessageType::PortInformation => {
                let info = PortInformationValue::parse(&mut msg_iter)?;
                PortInformation(info)
            }
            MessageType::PortModeInformation => {
                let info = PortModeInformationValue::parse(&mut msg_iter)?;
                PortModeInformation(info)
            }
            MessageType::PortValueSingle => {
                let value = PortValueSingleFormat::parse(&mut msg_iter)?;
                PortValueSingle(value)
            }
            MessageType::PortValueCombinedmode => {
                let value = PortValueCombinedFormat::parse(&mut msg_iter)?;
                PortValueCombinedmode(value)
            }
            MessageType::PortInputFormatSingle => {
                let fmt = PortInputFormatSingleFormat::parse(&mut msg_iter)?;
                PortInputFormatSingle(fmt)
            }
            MessageType::PortInputFormatCombinedmode => {
                let fmt = PortInputFormatCombinedFormat::parse(&mut msg_iter)?;
                PortInputFormatCombinedmode(fmt)
            }
            MessageType::VirtualPortSetup => {
                let setup = VirtualPortSetupFormat::parse(&mut msg_iter)?;
                VirtualPortSetup(setup)
            }
            MessageType::PortOutputCommand => {
                let cmd = PortOutputCommandFormat::parse(&mut msg_iter)?;
                PortOutputCommand(cmd)
            }
            MessageType::PortOutputCommandFeedback => {
                let feedback =
                    PortOutputCommandFeedbackFormat::parse(&mut msg_iter)?;
                PortOutputCommandFeedback(feedback)
            }
        })
    }

    /// Map from our enum members to MessageType values
    pub fn message_type(&self) -> u8 {
        // eww
        use NotificationMessage::*;
        (match self {
            HubProperties(_) => MessageType::HubProperties,
            HubActions(_) => MessageType::HubActions,
            HubAlerts(_) => MessageType::HubAlerts,
            HubAttachedIo(_) => MessageType::HubAttachedIo,
            GenericErrorMessages(_) => MessageType::GenericErrorMessages,
            HwNetworkCommands(_) => MessageType::HwNetworkCommands,
            FwUpdateGoIntoBootMode(_) => MessageType::FwUpdateGoIntoBootMode,
            FwUpdateLockMemory(_) => MessageType::FwUpdateLockMemory,
            FwUpdateLockStatusRequest => MessageType::FwUpdateLockStatusRequest,
            FwLockStatus(_) => MessageType::FwLockStatus,
            PortInformationRequest(_) => MessageType::PortInformationRequest,
            PortModeInformationRequest(_) => {
                MessageType::PortModeInformationRequest
            }
            PortInputFormatSetupSingle(_) => {
                MessageType::PortInputFormatSetupSingle
            }
            PortInputFormatSetupCombinedmode(_) => {
                MessageType::PortInputFormatSetupCombinedmode
            }
            PortInformation(_) => MessageType::PortInformation,
            PortModeInformation(_) => MessageType::PortModeInformation,
            PortValueSingle(_) => MessageType::PortValueSingle,
            PortValueCombinedmode(_) => MessageType::PortValueCombinedmode,
            PortInputFormatSingle(_) => MessageType::PortInputFormatSingle,
            PortInputFormatCombinedmode(_) => {
                MessageType::PortInputFormatCombinedmode
            }
            VirtualPortSetup(_) => MessageType::VirtualPortSetup,
            PortOutputCommand(_) => MessageType::PortOutputCommand,
            PortOutputCommandFeedback(_) => {
                MessageType::PortOutputCommandFeedback
            }
        }) as u8
    }

    fn validate_length<'a>(
        mut msg: impl Iterator<Item = &'a u8>,
        supplied: usize,
    ) -> Result<()> {
        let calculated = Self::length(&mut msg)?;
        if calculated != supplied {
            Err(Error::ParseError(format!(
                "Length mismatch {} != {}",
                calculated, supplied
            )))
        } else {
            Ok(())
        }
    }

    pub fn length<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<usize> {
        let first = next!(msg);

        let length = if first & 0x80 == 0x00 {
            // high bit not set - length is one byte
            (first & 0x7f) as usize
        } else {
            // high bit set - length is both bytes with a bit missing
            let second = next!(msg); // only advance if needed
            eprintln!("second: {:x}", second);
            ((second as usize) << 7) | ((first & 0x7f) as usize)
        };

        Ok(length)
    }

    /// ChkSum = PayLoad\[0\] ^ … PayLoad\[n\] ^ 0xFF
    pub fn checksum(buf: &[u8]) -> u8 {
        buf.iter().fold(0xff, |acc, x| acc ^ x)
    }

    /// Serialise a notification message into a Vec<u8>
    /// TODO no alloc
    pub fn serialise(&self) -> Vec<u8> {
        use NotificationMessage::*;

        let mut ser = match self {
            HubProperties(_) => todo!(),
            // HubProperties(msg) => msg.serialise(),
            HubActions(_) => todo!(),
            HubAlerts(_) => todo!(),
            HwNetworkCommands(_) => todo!(),
            FwUpdateGoIntoBootMode(_) => todo!(),
            FwUpdateLockMemory(_) => todo!(),
            FwUpdateLockStatusRequest => todo!(),
            PortInformationRequest(msg) => msg.serialise(),
            PortModeInformationRequest(msg) => msg.serialise(),
            PortInputFormatSetupSingle(msg) => msg.serialise(),
            PortInputFormatSetupCombinedmode(_) => todo!(),
            VirtualPortSetup(_) => todo!(),
            PortOutputCommand(cmd) => cmd.serialise(),

            // Documentation unclear; marked as upstream only but desc says
            // it can be used to request data. 
            HubAttachedIo(_) => todo!(),

            // These are upstream only and shouldn't need serialisation
            GenericErrorMessages(_) => todo!(),
            FwLockStatus(_) => todo!(),
            PortInformation(_) => todo!(),
            PortModeInformation(_) => todo!(),
            PortValueSingle(_) => todo!(),
            PortValueCombinedmode(_) => todo!(),
            PortInputFormatSingle(_) => todo!(),
            PortInputFormatCombinedmode(_) => todo!(),
            PortOutputCommandFeedback(_) => todo!(),
        };
        ser[0] = ser.len() as u8;
        debug!("Serialised to: {:02x?}", ser);
        ser
    }
}
