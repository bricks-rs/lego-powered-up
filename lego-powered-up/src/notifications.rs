use crate::consts::*;
use anyhow::{bail, Context, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

macro_rules! ok {
    ($thing:expr) => {
        $thing.context("Cannot convert 'None'")?
    };
}

macro_rules! next {
    ($iter:ident) => {
        *$iter.next().context("Insufficient length")?
    };
}

pub const MAX_NAME_SIZE: usize = 14;

/// Message format:
/// HEAD |
///
///
/// HEAD = LENGTH | HUB_ID (IGNORE) | TYPE
/// * LENGTH: u7(8) or u16, see below. Total length of message
/// * HUB_ID = 0_u8
/// * TYPE: message type u8. Message types are in consts::MessageType
///
/// LENGTH
/// lengths 0-127 are encoded as u8
/// if MSB of first byte is SET then discard this bit and take the next
/// byte from the message right shifted by 7 and OR it onto the first byte
/// i.e. LEN = BYTE1 as u16 & 0x7F | (BYTE2 as u16 >> 7)
///
/// The conversion to/from u8 is a bit of a hack pending
/// "Allow arbitrary enums to have explicit discriminants"
/// https://github.com/rust-lang/rust/issues/60553
///
/// As it stands we have a horrendous bodge involving consts::MessageType.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum NotificationMessage {
    HubProperties(HubProperty),
    HubActions,
    HubAlerts,
    HubAttachedIo,
    GenericErrorMessages,
    HwNetworkCommands,
    FwUpdateGoIntoBootMode,
    FwUpdateLockMemory,
    FwUpdateLockStatusRequest,
    FwLockStatus,
    PortInformationRequest,
    PortModeInformationRequest,
    PortInputFormatSetupSingle,
    PortInputFormatSetupCombinedmode,
    PortInformation,
    PortModeInformation,
    PortValueSingle,
    PortValueCombinedmode,
    PortInputFormatSingle,
    PortInputFormatCombinedmode,
    VirtualPortSetup,
    PortOutputCommand,
    PortOutputCommandFeedback,
}

impl NotificationMessage {
    pub fn parse(msg: &[u8]) -> Result<Self> {
        use NotificationMessage::*;

        println!("NOTIFICATION: {:?}", msg);

        let mut msg_iter = msg.iter();

        // consume the length bytes
        Self::validate_length(&mut msg_iter, msg.len())?;

        let _hub_id = next!(msg_iter);

        let message_type = ok!(MessageType::from_u8(next!(msg_iter)));

        eprintln!("Identified message type: {:?}", message_type);

        Ok(match message_type {
            MessageType::HubProperties => {
                let props = HubProperty::parse(&mut msg_iter)?;
                HubProperties(props)
            }
            _ => todo!(),
        })
    }

    pub fn message_type(&self) -> u8 {
        // eww
        use NotificationMessage::*;
        (match self {
            HubProperties(_) => MessageType::HubProperties,
            HubActions => MessageType::HubActions,
            HubAlerts => MessageType::HubAlerts,
            HubAttachedIo => MessageType::HubAttachedIo,
            GenericErrorMessages => MessageType::GenericErrorMessages,
            HwNetworkCommands => MessageType::HwNetworkCommands,
            FwUpdateGoIntoBootMode => MessageType::FwUpdateGoIntoBootMode,
            FwUpdateLockMemory => MessageType::FwUpdateLockMemory,
            FwUpdateLockStatusRequest => MessageType::FwUpdateLockStatusRequest,
            FwLockStatus => MessageType::FwLockStatus,
            PortInformationRequest => MessageType::PortInformationRequest,
            PortModeInformationRequest => {
                MessageType::PortModeInformationRequest
            }
            PortInputFormatSetupSingle => {
                MessageType::PortInputFormatSetupSingle
            }
            PortInputFormatSetupCombinedmode => {
                MessageType::PortInputFormatSetupCombinedmode
            }
            PortInformation => MessageType::PortInformation,
            PortModeInformation => MessageType::PortModeInformation,
            PortValueSingle => MessageType::PortValueSingle,
            PortValueCombinedmode => MessageType::PortValueCombinedmode,
            PortInputFormatSingle => MessageType::PortInputFormatSingle,
            PortInputFormatCombinedmode => {
                MessageType::PortInputFormatCombinedmode
            }
            VirtualPortSetup => MessageType::VirtualPortSetup,
            PortOutputCommand => MessageType::PortOutputCommand,
            PortOutputCommandFeedback => MessageType::PortOutputCommandFeedback,
        }) as u8
    }

    fn validate_length<'a>(
        mut msg: impl Iterator<Item = &'a u8>,
        supplied: usize,
    ) -> Result<()> {
        let calculated = Self::length(&mut msg)?;
        if calculated != supplied {
            bail!("Length mismatch {} != {}", calculated, supplied);
        } else {
            Ok(())
        }
    }

    fn length<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<usize> {
        let first = next!(msg);

        eprintln!("first: {:x}", first);

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
}

#[derive(Clone, Debug)]
pub struct HubProperty {
    property: HubPropertyValue,
    operation: HubPropertyOperation,
}

impl HubProperty {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let property_int = next!(msg);
        let operation = ok!(HubPropertyOperation::from_u8(next!(msg)));
        let property = HubPropertyValue::parse(property_int, &mut msg)?;

        Ok(Self {
            operation,
            property,
        })
    }
}

#[derive(Clone, Debug)]
pub enum HubPropertyValue {
    AdvertisingName(Vec<u8>),
    Button(u8),
    FwVersion(i32),
    HwVersion(i32),
    Rssi(i8),
    BatteryVoltage(u8),
    BatteryType(HubBatteryType),
    ManufacturerName(Vec<u8>),
    RadioFirmwareVersion(Vec<u8>),
    LegoWirelessProtocolVersion(u16),
    SystemTypeId(u8),
    HwNetworkId(u8),
    PrimaryMacAddress([u8; 6]),
    SecondaryMacAddress,
    HardwareNetworkFamily(u8),
}

impl HubPropertyValue {
    pub fn parse<'a>(
        prop_type: u8,
        mut msg: impl Iterator<Item = &'a u8>,
    ) -> Result<Self> {
        use HubPropertyValue::*;
        let prop_type = ok!(HubPropertyReference::from_u8(prop_type));

        Ok(match prop_type {
            HubPropertyReference::AdvertisingName => {
                // name is the rest of the data
                let name = msg.copied().collect();

                AdvertisingName(name)
            }
            HubPropertyReference::Button => Button(next!(msg)),
            HubPropertyReference::FwVersion => {
                let bytes = [next!(msg), next!(msg), next!(msg), next!(msg)];
                let vers = i32::from_le_bytes(bytes);

                FwVersion(vers)
            }
            HubPropertyReference::HwVersion => {
                let bytes = [next!(msg), next!(msg), next!(msg), next!(msg)];
                let vers = i32::from_le_bytes(bytes);

                HwVersion(vers)
            }
            HubPropertyReference::Rssi => {
                let bytes = [next!(msg)];
                let rssi = i8::from_le_bytes(bytes);

                Rssi(rssi)
            }
            HubPropertyReference::BatteryVoltage => BatteryVoltage(next!(msg)),
            HubPropertyReference::BatteryType => {
                BatteryType(ok!(HubBatteryType::from_u8(next!(msg))))
            }
            HubPropertyReference::ManufacturerName => {
                let name = msg.copied().collect();

                ManufacturerName(name)
            }
            HubPropertyReference::RadioFirmwareVersion => {
                let vers = msg.copied().collect();

                RadioFirmwareVersion(vers)
            }
            HubPropertyReference::LegoWirelessProtocolVersion => {
                let bytes = [next!(msg), next!(msg)];
                let vers = u16::from_le_bytes(bytes);

                LegoWirelessProtocolVersion(vers)
            }
            HubPropertyReference::SystemTypeId => SystemTypeId(next!(msg)),
            HubPropertyReference::HwNetworkId => HwNetworkId(next!(msg)),
            HubPropertyReference::PrimaryMacAddress => {
                let mac = [
                    next!(msg),
                    next!(msg),
                    next!(msg),
                    next!(msg),
                    next!(msg),
                    next!(msg),
                ];
                PrimaryMacAddress(mac)
            }
            HubPropertyReference::SecondaryMacAddress => SecondaryMacAddress,
            HubPropertyReference::HardwareNetworkFamily => {
                HardwareNetworkFamily(next!(msg))
            }
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum HubBatteryType {
    Normal = 0x00,
    Rechargeable = 0x01,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_message() {
        let msg = &[15, 0, 4, 0, 1, 47, 0, 0, 16, 0, 0, 0, 16, 0, 0];

        let notif = NotificationMessage::parse(msg).unwrap();
    }

    #[test]
    fn message_length() {
        let test_cases = &[
            ([0x34, 0x00], 0x34),
            ([0x7f, 0x00], 0x7f),
            ([0b1000_0000, 0b0000_0001], 128),
            ([0b1000_0001, 0b0000_0001], 129),
            ([0b1000_0010, 0b0000_0001], 130),
        ];

        for case in test_cases {
            assert_eq!(
                NotificationMessage::length(case.0.iter()).unwrap(),
                case.1
            );
        }
    }
}
