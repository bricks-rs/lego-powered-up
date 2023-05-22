// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Parser and data structure for hub notification messages

use crate::consts::*;
use crate::error::{Error, OptionContext, Result};
use log::{debug, trace};
use lpu_macros::Parse;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

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

macro_rules! four_bytes {
    ($t:ty, $iter:ident) => {
        <$t>::from_le_bytes([
            next!($iter),
            next!($iter),
            next!($iter),
            next!($iter),
        ])
    };
}

macro_rules! two_bytes {
    ($t:ty, $iter:ident) => {
        <$t>::from_le_bytes([next!($iter), next!($iter)])
    };
}

macro_rules! next_i32 {
    ($iter:ident) => {
        four_bytes!(i32, $iter)
    };
}

macro_rules! next_u32 {
    ($iter:ident) => {
        four_bytes!(u32, $iter)
    };
}

macro_rules! next_f32 {
    ($iter:ident) => {
        four_bytes!(f32, $iter)
    };
}

macro_rules! next_u16 {
    ($iter:ident) => {
        two_bytes!(u16, $iter)
    };
}

macro_rules! next_i16 {
    ($iter:ident) => {
        two_bytes!(i16, $iter)
    };
}

macro_rules! next_i8 {
    ($iter:ident) => {
        i8::from_le_bytes([next!($iter)])
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
/// <https://github.com/rust-lang/rust/issues/60553>
///
/// As it stands we have a horrendous bodge involving consts::MessageType.
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

    fn length<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<usize> {
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
            HubAttachedIo(_) => todo!(),
            GenericErrorMessages(_) => todo!(),
            HwNetworkCommands(_) => todo!(),
            FwUpdateGoIntoBootMode(_) => todo!(),
            FwUpdateLockMemory(_) => todo!(),
            FwUpdateLockStatusRequest => todo!(),
            FwLockStatus(_) => todo!(),
            PortInformationRequest(_) => todo!(),
            PortModeInformationRequest(_) => {
                todo!()
            }
            PortInputFormatSetupSingle(msg) => msg.serialise(),
            PortInputFormatSetupCombinedmode(_) => {
                todo!()
            }
            PortInformation(_) => todo!(),
            PortModeInformation(_) => todo!(),
            PortValueSingle(_) => todo!(),
            PortValueCombinedmode(_) => todo!(),
            PortInputFormatSingle(_) => todo!(),
            PortInputFormatCombinedmode(_) => {
                todo!()
            }
            VirtualPortSetup(_) => todo!(),
            PortOutputCommand(cmd) => cmd.serialise(),
            PortOutputCommandFeedback(_) => {
                todo!()
            }
        };
        ser[0] = ser.len() as u8;
        debug!("Serialised to: {:02x?}", ser);
        ser
    }
}

/// The two modes by which Hub LED colours may be set
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HubLedMode {
    /// Colour may be set to one of a number of specific named colours
    Colour = 0x0,
    /// Colour may be set to any 12-bit RGB value
    Rgb = 0x01,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

pub struct HubPropertiesRequest {

}

#[derive(Clone, Debug, PartialEq, Eq)]
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
                let vers = next_i32!(msg);

                FwVersion(vers)
            }
            HubPropertyReference::HwVersion => {
                let vers = next_i32!(msg);

                HwVersion(vers)
            }
            HubPropertyReference::Rssi => {
                let bytes = [next!(msg)];
                let rssi = i8::from_le_bytes(bytes);

                Rssi(rssi)
            }
            HubPropertyReference::BatteryVoltage => BatteryVoltage(next!(msg)),
            HubPropertyReference::BatteryType => {
                BatteryType(ok!(HubBatteryType::parse(&mut msg)))
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
                let vers = next_u16!(msg);

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
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum HubBatteryType {
    Normal = 0x00,
    Rechargeable = 0x01,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum HubAction {
    SwitchOffHub = 0x01,
    Disconnect = 0x02,
    VccPortControlOn = 0x03,
    VccPortControlOff = 0x04,
    ActivateBusyIndication = 0x05,
    ResetBusyIndication = 0x06,
    Shutdown = 0x2F,
    HubWillSwitchOff = 0x30,
    HubWillDisconnect = 0x31,
    HubWillGoIntoBootMode = 0x32,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum AlertType {
    LowVoltage = 0x01,
    HighCurrent = 0x02,
    LowSignalStrength = 0x03,
    OverPowerCondition = 0x04,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttachedIo {
    pub port: u8,
    pub event: IoAttachEvent,
}

impl AttachedIo {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port = next!(msg);
        let event = IoAttachEvent::parse(&mut msg)?;
        Ok(Self { port, event })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IoAttachEvent {
    DetachedIo {
        io_type_id: IoTypeId,
    },
    AttachedIo {
        hw_rev: VersionNumber,
        fw_rev: VersionNumber,
    },
    AttachedVirtualIo {
        port_a: u8,
        port_b: u8,
    },
}

impl IoAttachEvent {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let event_type = ok!(Event::from_u8(next!(msg)));

        Ok(match event_type {
            Event::DetachedIo => {
                let io_type_id = ok!(IoTypeId::from_u16(next_u16!(msg)));
                IoAttachEvent::DetachedIo { io_type_id }
            }
            Event::AttachedIo => {
                let hw_rev = VersionNumber::parse(&mut msg)?;
                let fw_rev = VersionNumber::parse(&mut msg)?;
                IoAttachEvent::AttachedIo { hw_rev, fw_rev }
            }
            Event::AttachedVirtualIo => {
                let port_a = next!(msg);
                let port_b = next!(msg);
                IoAttachEvent::AttachedVirtualIo { port_a, port_b }
            }
        })
    }
}

/// One observed version number (for a large motor) is 0x1000002f,
/// for which the build number component is decidedly *not* valid BCD,
/// so instead for the build number we just take the two bytes and
/// store them unconverted. As long as the build is printed as hex every
/// time then no one will notice
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct VersionNumber {
    pub major: u8,
    pub minor: u8,
    pub bugfix: u8,
    pub build: u16,
}

impl VersionNumber {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        //let byte0 = next!(msg);
        //let byte1 = next!(msg);
        let build = next_u16!(msg);
        let byte2 = next!(msg);
        let byte3 = next!(msg);
        //trace!("Bytes: {:02x?}", [byte3, byte2, byte1, byte0]);

        let major = (byte3 & 0x70) >> 4;
        let minor = byte3 & 0x0f;
        // BCD
        let bugfix = (byte2 >> 4) * 10 + (byte2 & 0x0f);
        /*
        let build = (byte1 >> 4) as u16 * 1000
            + (byte1 & 0x0f) as u16 * 100
            + (byte0 >> 4) as u16 * 10
            + (byte0 & 0x0f) as u16;
            */

        Ok(Self {
            major,
            minor,
            bugfix,
            build,
        })
    }

    pub fn serialise(&self) -> Vec<u8> {
        let byte3 = (self.major << 4) | self.minor;
        let byte2 = ((self.bugfix / 10) << 4) | (self.bugfix % 10);
        /*
        let mut digits = [0_u8; 4];
        let mut build = self.build;
        for digit in digits.iter_mut() {
            *digit = (build % 10) as u8;
            build /= 10;
        }
        trace!("digits: {:02x?}", digits);

        let byte1 = (digits[3] << 4) | digits[2];
        let byte0 = (digits[1] << 4) | digits[0];
        */
        let byte1 = (self.build >> 8) as u8;
        let byte0 = self.build as u8;

        vec![byte0, byte1, byte2, byte3]
    }
}

impl Display for VersionNumber {
    fn fmt(
        &self,
        fmt: &mut fmt::Formatter,
    ) -> std::result::Result<(), fmt::Error> {
        write!(
            fmt,
            "{}.{}.{}.{:x}",
            self.major, self.minor, self.bugfix, self.build
        )
    }
}

impl Debug for VersionNumber {
    fn fmt(
        &self,
        fmt: &mut fmt::Formatter,
    ) -> std::result::Result<(), fmt::Error> {
        write!(fmt, "{}", self)
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum IoTypeId {
    Motor = 0x0001,
    SystemTrainMotor = 0x0002,
    Button = 0x0005,
    LedLight = 0x0008,
    Voltage = 0x0014,
    Current = 0x0015,
    PiezoToneSound = 0x0016,
    RgbLight = 0x0017,
    ExternalTiltSensor = 0x0022,
    MotionSensor = 0x0023,
    VisionSensor = 0x0025,
    ExternalMotor = 0x0026,
    InternalMotor = 0x0027,
    InternalTilt = 0x0028,
    TechnicHubGestSensor = 0x0036,
    TechnicHubAccelerometer = 0x0039,
    TechnicHubGyroSensor = 0x003a,
    TechnicHubTiltSensor = 0x003b,
    TechnicHubTemperatureSensor = 0x003c
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ErrorMessageFormat {
    command_type: u8,
    error_code: ErrorCode,
}

impl ErrorMessageFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        trace!("ErrorMessageFormat");
        let command_type = next!(msg);
        let error_code = ErrorCode::parse(&mut msg)?;
        Ok(Self {
            command_type,
            error_code,
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum ErrorCode {
    Ack = 0x01,
    Mack = 0x02,
    BufferOverflow = 0x03,
    Timeout = 0x04,
    CommandNotRecognized = 0x05,
    InvalidUse = 0x06,
    Overcurrent = 0x07,
    InternalError = 0x08,
}

/// @typedef HWNetWorkCommandType
/// @param {number} CONNECTION_REQUEST 0x02
/// @param {number} FAMILY_REQUEST 0x03
/// @param {number} FAMILY_SET 0x04
/// @param {number} JOIN_DENIED 0x05
/// @param {number} GET_FAMILY 0x06
/// @param {number} FAMILY 0x07
/// @param {number} GET_SUBFAMILY 0x08
/// @param {number} SUBFAMILY 0x09
/// @param {number} SUBFAMILY_SET 0x0A
/// @param {number} GET_EXTENDED_FAMILY 0x0B
/// @param {number} EXTENDED_FAMILY 0x0C
/// @param {number} EXTENDED_FAMILY_SET 0x0D
/// @param {number} RESET_LONG_PRESS_TIMING 0x0E
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#h-w-network-command-type>
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NetworkCommand {
    ConnectionRequest(ButtonState),
    FamilyRequest,
    FamilySet(NetworkFamily),
    JoinDenied(),
    GetFamily(),
    Family(NetworkFamily),
    GetSubfamily(),
    Subfamily(NetworkSubFamily),
    SubfamilySet(NetworkSubFamily),
    GetExtendedFamily(),
    ExtendedFamily {
        family: NetworkFamily,
        subfamily: NetworkSubFamily,
    },
    ExtendedFamilySet {
        family: NetworkFamily,
        subfamily: NetworkSubFamily,
    },
    ResetLongPressTiming(),
}

impl NetworkCommand {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use NetworkCommand::*;
        let command_type = ok!(HwNetworkCommandType::from_u8(next!(msg)));

        Ok(match command_type {
            HwNetworkCommandType::ConnectionRequest => {
                let button = ButtonState::parse(&mut msg)?;
                ConnectionRequest(button)
            }
            HwNetworkCommandType::FamilyRequest => FamilyRequest,
            HwNetworkCommandType::FamilySet => {
                let fam = NetworkFamily::parse(&mut msg)?;
                FamilySet(fam)
            }
            HwNetworkCommandType::JoinDenied => {
                todo!()
            }
            HwNetworkCommandType::GetFamily => {
                todo!()
            }
            HwNetworkCommandType::Family => {
                let fam = NetworkFamily::parse(&mut msg)?;
                Family(fam)
            }
            HwNetworkCommandType::GetSubfamily => {
                todo!()
            }
            HwNetworkCommandType::Subfamily => {
                let fam = NetworkSubFamily::parse(&mut msg)?;
                Subfamily(fam)
            }
            HwNetworkCommandType::SubfamilySet => {
                let fam = NetworkSubFamily::parse(&mut msg)?;
                SubfamilySet(fam)
            }
            HwNetworkCommandType::GetExtendedFamily => {
                todo!()
            }
            HwNetworkCommandType::ExtendedFamily => {
                // Bit 7 | sss | ffff
                let byte = next!(msg);
                let fam_byte = byte & 0x0f;
                let sub_bytes = (byte >> 4) & 0x7;
                let family = ok!(NetworkFamily::from_u8(fam_byte));
                let subfamily = ok!(NetworkSubFamily::from_u8(sub_bytes));
                ExtendedFamily { family, subfamily }
            }
            HwNetworkCommandType::ExtendedFamilySet => {
                // Bit 7 | sss | ffff
                let byte = next!(msg);
                let fam_byte = byte & 0x0f;
                let sub_bytes = (byte >> 4) & 0x7;
                let family = ok!(NetworkFamily::from_u8(fam_byte));
                let subfamily = ok!(NetworkSubFamily::from_u8(sub_bytes));
                ExtendedFamilySet { family, subfamily }
            }
            HwNetworkCommandType::ResetLongPressTiming => {
                todo!()
            }
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum ButtonState {
    Pressed = 2,
    Released = 0,
    Up = 1,
    Down = 255,
    Stop = 127,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum NetworkFamily {
    Green = 0x01,
    Yellow = 0x02,
    Red = 0x03,
    Blue = 0x04,
    Purple = 0x05,
    LightBlue = 0x06,
    Teal = 0x07,
    Pink = 0x08,
    White = 0x00,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum NetworkSubFamily {
    OneFlash = 0x01,
    TwoFlashes = 0x02,
    ThreeFlashes = 0x03,
    FourFlashes = 0x04,
    FiveFlashes = 0x05,
    SixFlashes = 0x06,
    SevenFlashes = 0x07,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum LockStatus {
    Ok = 0x00,
    NotLocked = 0xff,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InformationRequest {
    port_id: u8,
    information_type: InformationType,
}

impl InformationRequest {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let information_type = InformationType::parse(&mut msg)?;
        Ok(InformationRequest {
            port_id,
            information_type,
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum InformationType {
    PortValue = 0x00,
    ModeInfo = 0x01,
    PossibleModeCombinations = 0x02,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModeInformationRequest {
    port_id: u8,
    mode: u8,
    information_type: ModeInformationType,
}

impl ModeInformationRequest {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let mode = next!(msg);
        let information_type = ModeInformationType::parse(&mut msg)?;
        Ok(Self {
            port_id,
            mode,
            information_type,
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum ModeInformationType {
    Name = 0x00,
    Raw = 0x01,
    Pct = 0x02,
    Si = 0x03,
    Symbol = 0x04,
    Mapping = 0x05,
    UsedInternally = 0x06,
    MotorBias = 0x07,
    CapabilityBits = 0x08,
    ValueFormat = 0x80,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InputSetupSingle {
    pub(crate) port_id: u8,
    pub(crate) mode: u8,
    pub(crate) delta: u32,
    pub(crate) notification_enabled: bool,
}

impl InputSetupSingle {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let mode = next!(msg);
        let delta = next_u32!(msg);
        let notif_byte = next!(msg);
        let notification_enabled = match notif_byte {
            0x00 => false,
            0x01 => true,
            b => {
                return Err(Error::ParseError(format!(
                    "Invalid notification enabled state {:x}",
                    b
                )))
            }
        };
        Ok(Self {
            port_id,
            mode,
            delta,
            notification_enabled,
        })
    }

    pub fn serialise(&self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(&[
            0,
            0,
            MessageType::PortInputFormatSetupSingle as u8,
            self.port_id,
            self.mode,
        ]);
        msg.extend_from_slice(&self.delta.to_le_bytes());
        msg.push(self.notification_enabled as u8);
        msg
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InputSetupCombined {
    port_id: u8,
    subcommand: InputSetupCombinedSubcommand,
}

impl InputSetupCombined {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let subcommand = InputSetupCombinedSubcommand::parse(&mut msg)?;
        Ok(InputSetupCombined {
            port_id,
            subcommand,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputSetupCombinedSubcommand {
    SetModeanddatasetCombinations {
        combination_index: u8,
        mode_dataset: [u8; 8],
    },
    LockLpf2DeviceForSetup,
    UnlockAndStartMultiEnabled,
    UnlockAndStartMultiDisabled,
    NotUsed,
    ResetSensor,
}

impl InputSetupCombinedSubcommand {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use InputSetupCombinedSubcommand::*;

        let comm = ok!(PortInputFormatSetupSubCommand::from_u8(next!(msg)));
        Ok(match comm {
            PortInputFormatSetupSubCommand::SetModeanddatasetCombinations => {
                let combination_index = next!(msg);
                let mut mode_dataset = [0_u8; 8];
                for ele in mode_dataset.iter_mut() {
                    *ele = next!(msg);
                }
                SetModeanddatasetCombinations {
                    combination_index,
                    mode_dataset,
                }
            }
            PortInputFormatSetupSubCommand::LockLpf2DeviceForSetup => {
                LockLpf2DeviceForSetup
            }
            PortInputFormatSetupSubCommand::UnlockAndStartMultiEnabled => {
                UnlockAndStartMultiEnabled
            }

            PortInputFormatSetupSubCommand::UnlockAndStartMultiDisabled => {
                UnlockAndStartMultiDisabled
            }
            PortInputFormatSetupSubCommand::NotUsed => NotUsed,
            PortInputFormatSetupSubCommand::ResetSensor => ResetSensor,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortInformationValue {
    port_id: u8,
    information_type: PortInformationType,
}

impl PortInformationValue {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let information_type = PortInformationType::parse(&mut msg)?;
        Ok(Self {
            port_id,
            information_type,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortInformationType {
    ModeInfo {
        capabilities: PortCapabilities,
        mode_count: u8,
        input_modes: u16,
        output_modes: u16,
    },
    PossibleModeCombinations(Vec<u8>),
}

impl PortInformationType {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use PortInformationType::*;

        let mode = next!(msg);
        match mode {
            1 => {
                // Mode info
                let capabilities = PortCapabilities(next!(msg));
                let mode_count = next!(msg);
                let input_modes = next_u16!(msg);
                let output_modes = next_u16!(msg);
                Ok(ModeInfo {
                    capabilities,
                    mode_count,
                    input_modes,
                    output_modes,
                })
            }
            2 => {
                // possible mode combinations
                let combinations = msg.cloned().collect();
                Ok(PossibleModeCombinations(combinations))
            }
            m => Err(Error::ParseError(format!(
                "Invalid port information type {}",
                m
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortCapabilities(u8);
impl PortCapabilities {
    pub const LOGICAL_SYNCHRONIZABLE: u8 = 0b1000;
    pub const LOGICAL_COMBINABLE: u8 = 0b0100;
    pub const INPUT: u8 = 0b0010;
    pub const OUTPUT: u8 = 0b0001;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortModeInformationValue {
    port_id: u8,
    mode: u8,
    information_type: PortModeInformationType,
}

impl PortModeInformationValue {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let mode = next!(msg);
        let information_type = PortModeInformationType::parse(&mut msg)?;
        Ok(Self {
            port_id,
            mode,
            information_type,
        })
    }
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum PortModeInformationType {
    Name(Vec<u8>),
    RawRange {
        min: f32,
        max: f32,
    },
    PctRange {
        min: f32,
        max: f32,
    },
    SiRange {
        min: f32,
        max: f32,
    },
    Symbol(Vec<u8>),
    Mapping {
        input: MappingValue,
        output: MappingValue,
    },
    MotorBias(u8),
    CapabilityBits([u8; 6]),
    ValueFormat(ValueFormatType),
}

impl PortModeInformationType {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use PortModeInformationType::*;

        let info_type = next!(msg);
        Ok(match info_type {
            0 => {
                // name is the remainder of the message
                let name = msg.cloned().collect();
                Name(name)
            }
            1 => {
                // raw is two 32-bit floats
                let min = next_f32!(msg);
                let max = next_f32!(msg);
                RawRange { min, max }
            }
            2 => {
                // pct is two 32-bit floats
                let min = next_f32!(msg);
                let max = next_f32!(msg);
                PctRange { min, max }
            }
            3 => {
                // si is two 32-bit floats
                let min = next_f32!(msg);
                let max = next_f32!(msg);
                SiRange { min, max }
            }
            4 => {
                // symbol is rest of message
                let sym = msg.cloned().collect();
                Symbol(sym)
            }
            5 => {
                // mapping is officially a u16 but actually looks like
                // two u8 values (one for input bitflags and one for
                // output bitflags)
                let input = MappingValue(next!(msg));
                let output = MappingValue(next!(msg));
                Mapping { input, output }
            }
            7 => {
                // motor bias is a byte
                MotorBias(next!(msg))
            }
            8 => {
                // capability bits is a 48-wide bitfield that might be
                // documented Somewhere™
                let mut bits = [0_u8; 6];
                for ele in bits.iter_mut() {
                    *ele = next!(msg);
                }
                CapabilityBits(bits)
            }
            128 => {
                // value format is the struct format
                let number_of_datasets = next!(msg);
                let dataset_type = DatasetType::parse(&mut msg)?;
                let total_figures = next!(msg);
                let decimals = next!(msg);
                ValueFormat(ValueFormatType {
                    number_of_datasets,
                    dataset_type,
                    total_figures,
                    decimals,
                })
            }
            t => {
                return Err(Error::ParseError(format!(
                    "Invalid information type {}",
                    t
                )))
            }
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ValueFormatType {
    number_of_datasets: u8,
    dataset_type: DatasetType,
    total_figures: u8,
    decimals: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MappingValue(u8);
impl MappingValue {
    pub const SUPPORTS_NULL: u8 = 0b1000_0000;
    pub const SUPPORTS_FUNCTIONAL2: u8 = 0b0100_0000;
    pub const ABS: u8 = 0b0001_0000;
    pub const REL: u8 = 0b0000_1000;
    pub const DIS: u8 = 0b0000_0100;
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum DatasetType {
    Bits8 = 0b00,
    Bits16 = 0b01,
    Bits32 = 0b10,
    Float = 0b11,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TypedValue {
    Bits8(u8),
    Bits16(u16),
    Bits32(u32),
    Float(f32),
}

/// The PortValueSingleFormat is a list of port id & value pairs, except
/// that the values may be different lengths (u8, u16, u32, f32) depending
/// on the port configuration. For now we just save the payload and then
/// later on will provide a method to split it out into port-value pairs
/// based on a separate port type mapping
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortValueSingleFormat {
    values: Vec<u8>,
}

impl PortValueSingleFormat {
    pub fn parse<'a>(msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let values = msg.cloned().collect();
        Ok(PortValueSingleFormat { values })
    }

    pub fn process(&self, _type_mapping: ()) -> HashMap<u8, TypedValue> {
        unimplemented!()
    }
}

/// The PortValueCombinedFormat is some horrific set of pointers to
/// values we should already have cached elsewhere. For now we save the
/// raw data and leave parsing it for later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortValueCombinedFormat {
    port_id: u8,
    data: Vec<u8>,
}

impl PortValueCombinedFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let data = msg.cloned().collect();
        Ok(Self { port_id, data })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortInputFormatSingleFormat {
    pub port_id: u8,
    pub mode: u8,
    pub delta: u32,
    pub notification_enabled: bool,
}

impl PortInputFormatSingleFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let mode = next!(msg);
        let delta = next_u32!(msg);
        let notification_enabled = match next!(msg) {
            0 => Ok(false),
            1 => Ok(true),
            v => Err(Error::ParseError(format!(
                "Invalid notification enabled status {}",
                v
            ))),
        }?;
        Ok(Self {
            port_id,
            mode,
            delta,
            notification_enabled,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortInputFormatCombinedFormat {
    port_id: u8,
    control: u8,
    combination_index: u8,
    multi_update: bool,
    mode_dataset_combination_pointer: u16,
}

impl PortInputFormatCombinedFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let control = next!(msg);
        let combination_index = next!(msg);
        let multi_update = (control >> 7) != 0;
        let mode_dataset_combination_pointer = next_u16!(msg);

        Ok(Self {
            port_id,
            control,
            combination_index,
            multi_update,
            mode_dataset_combination_pointer,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VirtualPortSetupFormat {
    Disconnect { port_id: u8 },
    Connect { port_a: u8, port_b: u8 },
}

impl VirtualPortSetupFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use VirtualPortSetupFormat::*;
        match next!(msg) {
            0 => {
                // Disconnected
                let port_id = next!(msg);
                Ok(Disconnect { port_id })
            }
            1 => {
                // Connected
                let port_a = next!(msg);
                let port_b = next!(msg);
                Ok(Connect { port_a, port_b })
            }
            c => Err(Error::ParseError(format!(
                "Invalid virtual port subcommand {}",
                c
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortOutputCommandFormat {
    pub port_id: u8,
    pub startup_info: StartupInfo,
    pub completion_info: CompletionInfo,
    pub subcommand: PortOutputSubcommand,
}

impl PortOutputCommandFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let startup_and_command_byte = next!(msg);
        let startup_info =
            ok!(StartupInfo::from_u8((startup_and_command_byte & 0xf0) >> 4));
        let completion_info =
            ok!(CompletionInfo::from_u8(startup_and_command_byte & 0x0f));
        let subcommand = PortOutputSubcommand::parse(&mut msg)?;

        Ok(Self {
            port_id,
            startup_info,
            completion_info,
            subcommand,
        })
    }

    pub fn serialise(&self) -> Vec<u8> {
        use PortOutputSubcommand::*;
        match &self.subcommand {
            StartSpeed {
                speed,
                max_power,
                use_acc_profile,
                use_dec_profile,
            } => {
                let profile =
                    ((*use_acc_profile as u8) << 1) | (*use_dec_profile as u8);
                let speed = speed.to_le_bytes()[0];
                vec![
                    0, // len
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    self.port_id,
                    0x11,
                    0x01,
                    speed,
                    max_power.to_u8(),
                    profile,
                ]
            }
            WriteDirectModeData(data) => data.serialise(self),
            _ => todo!(),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum StartupInfo {
    BufferIfNecessary = 0b0000,
    ExecuteImmediately = 0b0001,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum CompletionInfo {
    NoAction = 0b0000,
    CommandFeedback = 0b0001,
}

impl StartupInfo {
    pub fn serialise(&self, completion: &CompletionInfo) -> u8 {
        ((*self as u8) << 4) | (*completion as u8)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortOutputSubcommand {
    /// This has a subcommand number and also a "writedirectmodedata"
    /// annotation so I have no idea where this really lives
    ///
    /// According to (*) it does live here
    /// (*) <https://github.com/LEGO/lego-ble-wireless-protocol-docs/issues/15>
    StartPower2 {
        power1: Power,
        power2: Power,
    },
    SetAccTime {
        time: i16,
        profile_number: i8,
    },
    SetDecTime {
        time: i16,
        profile_number: i8,
    },
    StartSpeed {
        speed: i8,
        max_power: Power,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeed2 {
        speed1: i8,
        speed2: i8,
        max_power: Power,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForTime {
        time: i16,
        speed: i8,
        max_power: Power,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForTime2 {
        time: i16,
        speed_l: i8,
        speed_r: i8,
        max_power: Power,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForDegrees {
        degrees: i32,
        speed: i8,
        max_power: Power,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForDegrees2 {
        degrees: i32,
        speed_l: i8,
        speed_r: i8,
        max_power: Power,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    GotoAbsolutePosition {
        abs_pos: i32,
        speed: i8,
        max_power: Power,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    GotoAbsolutePosition2 {
        abs_pos1: i32,
        abs_pos2: i32,
        speed: i8,
        max_power: Power,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    PresetEncoder2 {
        left_position: i32,
        right_position: i32,
    },
    WriteDirect(WriteDirectPayload),
    WriteDirectModeData(WriteDirectModeDataPayload),
}

impl PortOutputSubcommand {
    pub const POWER_FLOAT: i8 = 0;
    pub const POWER_BRAKE: i8 = 127;

    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use PortOutputSubcommand::*;

        let subcomm = next!(msg);
        trace!("Port output subcommand: {:x}", subcomm);
        Ok(match subcomm {
            0x02 => {
                // StartPower(Power1, Power2)
                let power1 = Power::parse(&mut msg)?;
                let power2 = Power::parse(&mut msg)?;
                StartPower2 { power1, power2 }
            }
            0x05 => {
                // SetAccTime(Time, ProfileNo)
                let time = next_i16!(msg);
                let profile_number = next_i8!(msg);
                SetAccTime {
                    time,
                    profile_number,
                }
            }
            0x06 => {
                // SetDecTime(Time, ProfileNo)
                let time = next_i16!(msg);
                let profile_number = next_i8!(msg);
                SetDecTime {
                    time,
                    profile_number,
                }
            }
            0x07 => {
                // StartSpeed(Speed, MaxPower, UseProfile)
                let speed = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                StartSpeed {
                    speed,
                    max_power,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x08 => {
                // StartSpeed(Speed1, Speed2, MaxPower, UseProfile)
                let speed1 = next_i8!(msg);
                let speed2 = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                StartSpeed2 {
                    speed1,
                    speed2,
                    max_power,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x09 => {
                // StartSpeedForTime (Time, Speed, MaxPower, EndState, UseProfile)
                let time = next_i16!(msg);
                let speed = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let end_state = EndState::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                StartSpeedForTime {
                    time,
                    speed,
                    max_power,
                    end_state,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x0a => {
                // StartSpeedForTime(Time, SpeedL, SpeedR, MaxPower, EndState,
                // UseProfile)
                let time = next_i16!(msg);
                let speed_l = next_i8!(msg);
                let speed_r = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let end_state = EndState::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                StartSpeedForTime2 {
                    time,
                    speed_l,
                    speed_r,
                    max_power,
                    end_state,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x0b => {
                // StartSpeedForDegrees (Degrees, Speed, MaxPower, EndState,
                // UseProfile)
                let degrees = next_i32!(msg);
                let speed = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let end_state = EndState::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                StartSpeedForDegrees {
                    degrees,
                    speed,
                    max_power,
                    end_state,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x0c => {
                // StartSpeedForDegrees2 (Degrees, SpeedL, SpeedR, MaxPower,
                // EndState, UseProfile)
                let degrees = next_i32!(msg);
                let speed_l = next_i8!(msg);
                let speed_r = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let end_state = EndState::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                StartSpeedForDegrees2 {
                    degrees,
                    speed_l,
                    speed_r,
                    max_power,
                    end_state,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x0d => {
                // GotoAbsolutePosition(AbsPos, Speed, MaxPower, EndState,
                // UseProfile)
                let abs_pos = next_i32!(msg);
                let speed = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let end_state = EndState::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                GotoAbsolutePosition {
                    abs_pos,
                    speed,
                    max_power,
                    end_state,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x0e => {
                // GotoAbsolutePosition(AbsPos1, AbsPos2, Speed, MaxPower,
                // EndState, UseProfile)
                let abs_pos1 = next_i32!(msg);
                let abs_pos2 = next_i32!(msg);
                let speed = next_i8!(msg);
                let max_power = Power::parse(&mut msg)?;
                let end_state = EndState::parse(&mut msg)?;
                let use_prof = next!(msg);
                let use_acc_profile = (use_prof & 0x01) != 0;
                let use_dec_profile = (use_prof & 0x02) != 0;
                GotoAbsolutePosition2 {
                    abs_pos1,
                    abs_pos2,
                    speed,
                    max_power,
                    end_state,
                    use_acc_profile,
                    use_dec_profile,
                }
            }
            0x14 => {
                // PresetEncoder(LeftPosition, RightPosition)
                let left_position = next_i32!(msg);
                let right_position = next_i32!(msg);
                PresetEncoder2 {
                    left_position,
                    right_position,
                }
            }
            50 => {
                // WriteDirect(Byte[0],Byte[0 + n])
                let data = WriteDirectPayload::parse(&mut msg)?;
                WriteDirect(data)
            }
            51 => {
                // WriteDirectModeData(Mode, PayLoad[0] PayLoad [0 + n]
                let data = WriteDirectModeDataPayload::parse(&mut msg)?;
                WriteDirectModeData(data)
            }
            c => {
                return Err(Error::ParseError(format!(
                    "Invalid port output subcommand {}",
                    c
                )))
            }
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Power {
    Cw(u8),
    Ccw(u8),
    Float,
    Brake,
}

impl Power {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let val = next_i8!(msg);
        Power::from_i8(val)
    }

    pub fn to_u8(&self) -> u8 {
        use Power::*;
        let integer: i8 = match self {
            Float => 0,
            Brake => 127,
            Cw(p) => *p as i8,
            Ccw(p) => -(*p as i8),
        };
        integer.to_le_bytes()[0]
    }

    pub fn from_i8(val: i8) -> Result<Self> {
        use Power::*;
        match val {
            0 => Ok(Float),
            127 => Ok(Brake),
            p if (1..=100).contains(&p) => Ok(Cw(p as u8)),
            p if (-100..=-1).contains(&p) => Ok(Ccw((-p) as u8)),
            p => Err(Error::ParseError(format!(
                "Invalid value for power: {}",
                p
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteDirectPayload {
    TiltFactoryCalibration {
        orientation: CalibrationOrientation,
        pass_code: String,
    },
    HardwareReset,
}

impl WriteDirectPayload {
    pub fn parse<'a>(_msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        todo!()
    }
}

#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum CalibrationOrientation {
    LayingFlat = 1,
    Standing = 2,
}

impl CalibrationOrientation {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        Ok(ok!(Self::from_i8(next_i8!(msg))))
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum EndState {
    Float = 0,
    Hold = 126,
    Brake = 127,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteDirectModeDataPayload {
    StartPower(Power),
    PresetEncoder(i32),
    TiltImpactPreset(i32),
    TiltConfigOrientation(Orientation),
    TiltConfigImpact {
        impact_threshold: i8,
        bump_holdoff: i8,
    },
    SetRgbColorNo(i8),
    SetRgbColors {
        red: u8,
        green: u8,
        blue: u8,
    },
}

impl WriteDirectModeDataPayload {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use WriteDirectModeDataPayload::*;

        let mode = next!(msg);
        Ok(match mode {
            0x01 => {
                // StartPower(Power)
                let power = Power::parse(&mut msg)?;
                StartPower(power)
            }
            0x02 => {
                // PresetEncoder(Position)
                let position = next_i32!(msg);
                PresetEncoder(position)
            }
            0x03 => {
                // TiltImpactPreset(PresetValue)
                // "Mode 3 impact counts"
                let preset_value = next_i32!(msg);
                TiltImpactPreset(preset_value)
            }
            0x05 => {
                // TiltConfigOrientation(Orientation)
                // "Mode 5"
                let orientation = Orientation::parse(&mut msg)?;
                TiltConfigOrientation(orientation)
            }
            0x06 => {
                // TiltConfigImpact(ImpactThreshold, BumpHoldoff)
                // "Mode 6"
                let impact_threshold = next_i8!(msg);
                let bump_holdoff = next_i8!(msg);
                TiltConfigImpact {
                    impact_threshold,
                    bump_holdoff,
                }
            }
            0x08 => {
                // SetRgbColorNo(ColorNo)
                let col = next_i8!(msg);
                SetRgbColorNo(col)
            }
            0x09 => {
                // SetRgbColors(RedColor, GreenColor, BlueColor)
                let red = next!(msg);
                let green = next!(msg);
                let blue = next!(msg);
                SetRgbColors { red, green, blue }
            }
            m => {
                return Err(Error::ParseError(format!(
                    "Invalid write direct mode {}",
                    m
                )))
            }
        })
    }

    pub fn serialise(&self, meta: &PortOutputCommandFormat) -> Vec<u8> {
        use WriteDirectModeDataPayload::*;
        match self {
            SetRgbColors { red, green, blue } => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    HubLedMode::Rgb as u8,
                    *red,
                    *green,
                    *blue,
                ]
            }
            StartPower(p) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                let power = p.to_u8();
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    0x00, // magic value from docs
                    power,
                ]
            }
            _ => todo!(),
        }
    }
}

#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum Orientation {
    Bottom = 0,
    Front = 1,
    Back = 2,
    Left = 3,
    Right = 4,
    Top = 5,
    UseActualAsBottomReference = 6,
}

impl Orientation {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        Ok(ok!(Self::from_i8(next_i8!(msg))))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PortOutputCommandFeedbackFormat {
    msg1: FeedbackMessage,
    msg2: Option<FeedbackMessage>,
    msg3: Option<FeedbackMessage>,
}

impl PortOutputCommandFeedbackFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let msg1 = FeedbackMessage::parse(&mut msg)?;
        let msg2 = FeedbackMessage::parse(&mut msg).ok();
        let msg3 = FeedbackMessage::parse(&mut msg).ok();
        Ok(PortOutputCommandFeedbackFormat { msg1, msg2, msg3 })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FeedbackMessage {
    port_id: u8,
    empty_cmd_in_progress: bool,
    empty_cmd_completed: bool,
    discarded: bool,
    idle: bool,
    busy_full: bool,
}

impl FeedbackMessage {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let bitfields = next!(msg);
        let empty_cmd_in_progress = (bitfields & 0x01) != 0;
        let empty_cmd_completed = (bitfields & 0x02) != 0;
        let discarded = (bitfields & 0x04) != 0;
        let idle = (bitfields & 0x08) != 0;
        let busy_full = (bitfields & 0x10) != 0;
        Ok(FeedbackMessage {
            port_id,
            empty_cmd_in_progress,
            empty_cmd_completed,
            discarded,
            idle,
            busy_full,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::LevelFilter;

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter(None, LevelFilter::Trace)
            .try_init();
    }

    #[test]
    fn attach_io_message() {
        init();
        let msgs: &[&[u8]] = &[
            &[15, 0, 4, 0, 1, 47, 0, 0, 16, 0, 0, 0, 16, 0, 0],
            &[15, 0, 4, 50, 1, 23, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 59, 1, 21, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 60, 1, 20, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 61, 1, 60, 0, 0, 0, 0, 16, 0, 0, 0, 16],
            &[15, 0, 4, 96, 1, 60, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 97, 1, 57, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 98, 1, 58, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 99, 1, 59, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            &[15, 0, 4, 100, 1, 54, 0, 1, 0, 0, 0, 1, 0, 0, 0],
        ];
        for msg in msgs {
            let notif = NotificationMessage::parse(msg).unwrap();
            if let NotificationMessage::HubAttachedIo(_) = notif {
                // OK
            } else {
                panic!("wrong type");
            }
        }
    }

    #[test]
    fn error_message() {
        init();
        let msgs: &[&[u8]] = &[&[5, 0, 5, 17, 5]];
        for msg in msgs {
            let _notif = NotificationMessage::parse(msg).unwrap();
        }
    }

    /*#[test]
    fn write_direct() {
        init();
        let msgs: &[&[u8]] = &[&[9, 0, 129, 81, 50, 1, 0, 255, 0]];
        for msg in msgs {
            let _notif = NotificationMessage::parse(msg).unwrap();
        }
    }*/

    #[test]
    fn message_length() {
        init();
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

    #[test]
    fn serialise_write_direct() {
        /* Hub LED, from the arduino lib:
        byte port = getPortForDeviceType((byte)DeviceType::HUB_LED);
        byte setRGBMode[8] = {0x41, port, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00};
        WriteValue(setRGBMode, 8);
        byte setRGBColor[8] = {0x81, port, 0x11, 0x51, 0x01, red, green, blue};
        WriteValue(setRGBColor, 8);
        // WriteValue adds the length header and hub id = 0 header
        // https://github.com/corneliusmunz/legoino/blob/master/src/Lpf2Hub.cpp#L952
        */
        init();
        let startup_info = StartupInfo::ExecuteImmediately;
        let completion_info = CompletionInfo::CommandFeedback;

        let subcommand = PortOutputSubcommand::WriteDirectModeData(
            WriteDirectModeDataPayload::SetRgbColors {
                red: 0x12,
                green: 0x34,
                blue: 0x56,
            },
        );

        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: 50,
                startup_info,
                completion_info,
                subcommand,
            });

        let serialised = msg.serialise();
        let correct =
            &mut [0_u8, 0, 0x81, 50, 0x11, 0x51, 0x01, 0x12, 0x34, 0x56];
        correct[0] = correct.len() as u8;

        assert_eq!(&serialised, correct);
    }

    #[test]
    fn port_input_format_setup_single() {
        init();

        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id: 50,
                mode: 0x01,
                delta: 0x00000001,
                notification_enabled: false,
            });

        let serialised = msg.serialise();
        let correct =
            &mut [0_u8, 0, 0x41, 50, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00];
        correct[0] = correct.len() as u8;

        assert_eq!(&serialised, correct);
    }

    #[test]
    fn version_number() {
        init();
        // first test case from documentation
        // remainder from observed hardware
        let test_cases: &[(i32, VersionNumber)] = &[
            (
                0x17371510,
                VersionNumber {
                    major: 1,
                    minor: 7,
                    bugfix: 37,
                    build: 0x1510,
                },
            ),
            (
                268435503,
                VersionNumber {
                    major: 1,
                    minor: 0,
                    bugfix: 0,
                    build: 0x2f,
                },
            ),
            (
                268435456,
                VersionNumber {
                    major: 1,
                    minor: 0,
                    bugfix: 0,
                    build: 0,
                },
            ),
            (
                23,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 23,
                },
            ),
            (
                21,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 21,
                },
            ),
            (
                20,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 20,
                },
            ),
            (
                60,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 60,
                },
            ),
            (
                4096,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 0,
                    build: 4096,
                },
            ),
            (
                65596,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 60,
                },
            ),
            (
                65536,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 0,
                },
            ),
            (
                65593,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 57,
                },
            ),
            (
                65594,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 58,
                },
            ),
            (
                65595,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 59,
                },
            ),
            (
                65590,
                VersionNumber {
                    major: 0,
                    minor: 0,
                    bugfix: 1,
                    build: 54,
                },
            ),
        ];

        for (number, correct) in test_cases {
            eprintln!("\ntest case: {:08x} - {}", number, correct);
            let parsed =
                VersionNumber::parse(number.to_le_bytes().iter()).unwrap();
            assert_eq!(parsed, *correct);

            let serialised = correct.serialise();
            eprintln!("serialised: {:02x?}", serialised);
            eprintln!("correct LE: {:02x?}", number.to_le_bytes());
            assert_eq!(serialised, &number.to_le_bytes());
        }
    }

    #[test]
    fn motor_set_speed() {
        init();
        let subcommand = PortOutputSubcommand::StartSpeed {
            speed: 0x12,
            max_power: Power::Cw(0x34),
            use_acc_profile: true,
            use_dec_profile: true,
        };
        let msg =
            NotificationMessage::PortOutputCommand(PortOutputCommandFormat {
                port_id: 1,
                startup_info: StartupInfo::ExecuteImmediately,
                completion_info: CompletionInfo::NoAction,
                subcommand,
            });
        let serialised = msg.serialise();
        let correct = &mut [0, 0, 0x81, 1, 0x11, 0x01, 0x12, 0x34, 0x03];
        correct[0] = correct.len() as u8;

        assert_eq!(&serialised, correct);
    }
}
