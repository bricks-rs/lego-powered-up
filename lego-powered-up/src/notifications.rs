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

pub use self::message::NotificationMessage;
pub mod message;

pub use self::macros::*;
#[macro_use]
pub mod macros;

pub const MAX_NAME_SIZE: usize = 14;

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
    pub(crate) property: HubPropertyValue,
    pub(crate) operation: HubPropertyOperation,
    pub(crate) reference: HubPropertyRef,
}

impl HubProperty {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let property_int = next!(msg);
        let operation = ok!(HubPropertyOperation::from_u8(next!(msg)));
        let property = HubPropertyValue::parse(property_int, &mut msg)?;
        let reference = match property {
            HubPropertyValue::AdvertisingName(_) => {
                HubPropertyRef::AdvertisingName
            }
            HubPropertyValue::Button(_) => HubPropertyRef::Button,
            HubPropertyValue::FwVersion(_) => HubPropertyRef::FwVersion,
            HubPropertyValue::HwVersion(_) => HubPropertyRef::HwVersion,
            HubPropertyValue::Rssi(_) => HubPropertyRef::Rssi,
            HubPropertyValue::BatteryVoltage(_) => {
                HubPropertyRef::BatteryVoltage
            }
            HubPropertyValue::BatteryType(_) => HubPropertyRef::BatteryType,
            HubPropertyValue::ManufacturerName(_) => {
                HubPropertyRef::ManufacturerName
            }
            HubPropertyValue::RadioFirmwareVersion(_) => {
                HubPropertyRef::RadioFirmwareVersion
            }
            HubPropertyValue::LegoWirelessProtocolVersion(_) => {
                HubPropertyRef::LegoWirelessProtocolVersion
            }
            HubPropertyValue::SystemTypeId(_) => HubPropertyRef::SystemTypeId,
            HubPropertyValue::HwNetworkId(_) => HubPropertyRef::HwNetworkId,
            HubPropertyValue::PrimaryMacAddress(_) => {
                HubPropertyRef::PrimaryMacAddress
            }
            HubPropertyValue::SecondaryMacAddress => {
                HubPropertyRef::SecondaryMacAddress
            }
            HubPropertyValue::HardwareNetworkFamily(_) => {
                HubPropertyRef::HardwareNetworkFamily
            }
        };

        Ok(Self {
            reference,
            operation,
            property,
        })
    }
    pub fn serialise(&self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(&[
            0,
            0,
            MessageType::HubProperties as u8,
            // prop_ref,
            self.reference as u8,
            self.operation as u8,
        ]);

        msg
    }
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
        let prop_type = ok!(HubPropertyRef::from_u8(prop_type));

        Ok(match prop_type {
            HubPropertyRef::AdvertisingName => {
                // name is the rest of the data
                let name = msg.copied().collect();

                AdvertisingName(name)
            }
            HubPropertyRef::Button => Button(next!(msg)),
            HubPropertyRef::FwVersion => {
                let vers = next_i32!(msg);

                FwVersion(vers)
            }
            HubPropertyRef::HwVersion => {
                let vers = next_i32!(msg);

                HwVersion(vers)
            }
            HubPropertyRef::Rssi => {
                let bytes = [next!(msg)];
                let rssi = i8::from_le_bytes(bytes);

                Rssi(rssi)
            }
            HubPropertyRef::BatteryVoltage => BatteryVoltage(next!(msg)),
            HubPropertyRef::BatteryType => {
                BatteryType(ok!(HubBatteryType::parse(&mut msg)))
            }
            HubPropertyRef::ManufacturerName => {
                let name = msg.copied().collect();

                ManufacturerName(name)
            }
            HubPropertyRef::RadioFirmwareVersion => {
                let vers = msg.copied().collect();

                RadioFirmwareVersion(vers)
            }
            HubPropertyRef::LegoWirelessProtocolVersion => {
                let vers = next_u16!(msg);

                LegoWirelessProtocolVersion(vers)
            }
            HubPropertyRef::SystemTypeId => SystemTypeId(next!(msg)),
            HubPropertyRef::HwNetworkId => HwNetworkId(next!(msg)),
            HubPropertyRef::PrimaryMacAddress => {
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
            HubPropertyRef::SecondaryMacAddress => SecondaryMacAddress,
            HubPropertyRef::HardwareNetworkFamily => {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HubActionRequest {
    pub(crate) action_type: HubAction,
}

impl HubActionRequest {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let action_type = HubAction::parse(&mut msg)?;
        Ok(HubActionRequest { action_type })
    }
    pub fn serialise(&self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(&[
            0,
            0,
            MessageType::HubActions as u8,
            self.action_type as u8,
        ]);
        msg
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum AlertType {
    LowVoltage = 0x01,
    HighCurrent = 0x02,
    LowSignalStrength = 0x03,
    OverPowerCondition = 0x04,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum AlertOperation {
    EnableUpdates = 0x01,
    DisableUpdates = 0x02,
    RequestUpdate = 0x03,
    Update = 0x04,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse)]
pub enum AlertPayload {
    StatusOk = 0x00,
    Alert = 0xFF,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HubAlert {
    pub(crate) alert_type: AlertType,
    pub(crate) operation: AlertOperation,
    pub(crate) payload: AlertPayload,
}

impl HubAlert {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let alert_type = AlertType::parse(&mut msg)?;
        let operation = AlertOperation::parse(&mut msg)?;
        let payload = AlertPayload::parse(&mut msg)?;
        Ok(HubAlert {
            alert_type,
            operation,
            payload,
        })
    }
    pub fn serialise(&self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(&[
            0,
            0,
            MessageType::HubAlerts as u8,
            self.alert_type as u8,
            self.operation as u8,
        ]);
        msg
    }
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
        // io_type_id: IoTypeId,        //Not included in detached event
    },
    AttachedIo {
        io_type_id: IoTypeId,
        hw_rev: VersionNumber,
        fw_rev: VersionNumber,
    },
    AttachedVirtualIo {
        io_type_id: IoTypeId,
        port_a: u8,
        port_b: u8,
    },
}

impl IoAttachEvent {
    // Note: Returns "NoneError("Cannot convert 'None'")"
    // if incoming IoTypeId-value is not in enum IoTypeId.
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let event_type = ok!(Event::from_u8(next!(msg)));

        Ok(match event_type {
            Event::DetachedIo => {
                // let io_type_id = ok!(IoTypeId::from_u16(next_u16!(msg)));
                IoAttachEvent::DetachedIo {}
            }

            Event::AttachedIo => {
                let io_type_id = ok!(IoTypeId::from_u16(next_u16!(msg)));
                let hw_rev = VersionNumber::parse(&mut msg)?;
                let fw_rev = VersionNumber::parse(&mut msg)?;
                IoAttachEvent::AttachedIo {
                    io_type_id,
                    hw_rev,
                    fw_rev,
                }
            }
            Event::AttachedVirtualIo => {
                let io_type_id = ok!(IoTypeId::from_u16(next_u16!(msg)));
                let port_a = next!(msg);
                let port_b = next!(msg);
                IoAttachEvent::AttachedVirtualIo {
                    io_type_id,
                    port_a,
                    port_b,
                }
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
    pub(crate) port_id: u8,
    pub(crate) information_type: InformationType,
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
    pub fn serialise(&self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(&[
            0,
            0,
            MessageType::PortInformationRequest as u8,
            self.port_id,
            self.information_type as u8,
        ]);
        msg
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
    pub(crate) port_id: u8,
    pub(crate) mode: u8,
    pub(crate) information_type: ModeInformationType,
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
    pub fn serialise(&self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(10);
        msg.extend_from_slice(&[
            0,
            0,
            MessageType::PortModeInformationRequest as u8,
            self.port_id,
            self.mode,
            self.information_type as u8,
        ]);
        msg
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
    pub(crate) port_id: u8,
    pub(crate) subcommand: InputSetupCombinedSubcommand,
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
    pub fn serialise(&self) -> Vec<u8> {
        use InputSetupCombinedSubcommand::*;
        match &self.subcommand {
            SetModeanddatasetCombinations {
                combination_index,
                mode_dataset,
            } => {
                let mut bytes = vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortInputFormatSetupCombined as u8,
                    // Command
                    self.port_id,
                    InputSetupCombinedSubcommandValue::SetModeanddatasetCombinations as u8,
                    // Subcommand payload
                    *combination_index,
                ];
                // Not sure why mode_dataset needs to be a [u8; 8] but changing it necessitates reworking
                // the parse function and possibly more. Workaround for now is to set unneeded values to
                // all 1's as a marker. Should be ok since no device probably has 128 modes and 128 datasets.
                let md = mode_dataset.as_slice();
                for val in md.iter() {
                    if *val == 255 {
                        break;
                    } else {
                        bytes.push(*val);
                    }
                }
                // bytes.extend_from_slice(mode_dataset.as_slice());
                bytes
            }
            LockLpf2DeviceForSetup {} => {
                vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortInputFormatSetupCombined as u8,
                    // Command
                    self.port_id,
                    InputSetupCombinedSubcommandValue::LockLpf2DeviceForSetup
                        as u8,
                ]
            }
            UnlockAndStartMultiEnabled {} => {
                vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortInputFormatSetupCombined as u8,
                    // Command
                    self.port_id,
                    InputSetupCombinedSubcommandValue::UnlockAndStartMultiEnabled as u8,
                ]
            }
            UnlockAndStartMultiDisabled {} => {
                vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortInputFormatSetupCombined as u8,
                    // Command
                    self.port_id,
                    InputSetupCombinedSubcommandValue::UnlockAndStartMultiDisabled as u8,
                ]
            }
            NotUsed {} => {
                vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortInputFormatSetupCombined as u8,
                    // Command
                    self.port_id,
                    InputSetupCombinedSubcommandValue::NotUsed as u8,
                ]
            }
            ResetSensor {} => {
                vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortInputFormatSetupCombined as u8,
                    // Command
                    self.port_id,
                    InputSetupCombinedSubcommandValue::ResetSensor as u8,
                ]
            }
        }
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
    pub port_id: u8,
    pub information_type: PortInformationType,
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
pub struct PortCapabilities(pub u8);
impl PortCapabilities {
    pub const LOGICAL_SYNCHRONIZABLE: u8 = 0b1000;
    pub const LOGICAL_COMBINABLE: u8 = 0b0100;
    pub const INPUT: u8 = 0b0010;
    pub const OUTPUT: u8 = 0b0001;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortModeInformationValue {
    pub port_id: u8,
    pub mode: u8,
    pub information_type: PortModeInformationType,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ValueFormatType {
    pub number_of_datasets: u8,
    pub dataset_type: DatasetType,
    pub total_figures: u8,
    pub decimals: u8,
}
impl fmt::Display for ValueFormatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>4} value(s) of {:<8}   Figures: {:<3} Decimals: {:<3}",
            self.number_of_datasets,
            self.dataset_type,
            self.total_figures,
            self.decimals
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MappingValue(pub u8);
impl MappingValue {
    pub const SUPPORTS_NULL: u8 = 0b1000_0000;
    pub const SUPPORTS_FUNCTIONAL2: u8 = 0b0100_0000;
    pub const ABS: u8 = 0b0001_0000;
    pub const REL: u8 = 0b0000_1000;
    pub const DIS: u8 = 0b0000_0100;
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Parse, Default)]
pub enum DatasetType {
    #[default]
    Bits8 = 0b00,
    Bits16 = 0b01,
    Bits32 = 0b10,
    Float = 0b11,
}
impl fmt::Display for DatasetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DatasetType::Bits8 => {
                write!(f, " 8 bit")
            }
            DatasetType::Bits16 => {
                write!(f, "16 bit")
            }
            DatasetType::Bits32 => {
                write!(f, "32 bit")
            }
            DatasetType::Float => {
                write!(f, "float ")
            }
        }
        // write!(f, "{}",
        // *self,
        // )
    }
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
/// based on a separate port type mapping.
///
/// Notes on Value Format
/// 1) The valuetypes are signed, i.e. the variants are i8, i16, i32, f32. This:
///    https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#port-value-single
///    says that the values are unsigned, but this is seemingly incorrect. Many sensors can
///    report negative values, as can be seen by requesting Port Mode Information::Raw range.
/// 2) The values are not a single value but an array, the length of which is given
///    by the "number_of_datasets"-member of Value Format.
///    ("Single" in PortValueSingle refers to single sensor mode, but single sensors can)  
///     and do provide provide array data, ex. color RGB or accelerometer XYZ-data.)
/// 3) There are some inconsistencies looking at port mode information:
///         HubLeds in RBG reports taking 8 bit values in the range 0-255, though this
///         doesn't concern the parser of incoming values. As regards sensors;
///         TechnicHubTiltSensor mode CFG, as well as MoveHubInternalTilt modes IM_CF
///         and CALIB: These all report that they will provide 8 bit values in
///         range 0-255.
///     But these are the only ones I've been able to find. On the whole it seems better
///     to correctly support the multitude of sensors and modes.
///

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct PortValueSingleFormat {
//     pub port_id: u8,
//     pub data: Vec<u8>,
// }
// impl PortValueSingleFormat {
//     pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
//         // let values = msg.cloned().collect();
//         // Ok(PortValueSingleFormat { values })
//         let port_id = next!(msg);
//         let data = msg.cloned().collect();
//         Ok(Self { port_id, data })
//     }

//     pub fn process(&self, _type_mapping: ()) -> HashMap<u8, TypedValue> {
//         unimplemented!()
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortValueSingleFormat {
    pub port_id: u8,
    pub data: Vec<i8>,
}
impl PortValueSingleFormat {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port_id = next!(msg);
        let data = msg.cloned().map(|x| x as i8).collect();
        Ok(Self { port_id, data })
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
    pub port_id: u8,
    pub data: Vec<u8>,
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

        // let combination_index = next!(msg);  // combination index is part of control byte, not separate byte.
        // This caused function to fail with "NoneError: Insufficient length"
        let combination_index: u8 = 0; // Set to 0 for now, figure out how to get from control byte later

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
        // use crate::consts::PortOutputSubCommandValue;
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
                let max_power = max_power.to_le_bytes()[0];
                vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortOutputCommand as u8,
                    // Command
                    self.port_id,
                    0x11, // 0001 Execute immediately, 0001 Command feedback
                    PortOutputSubCommandValue::StartSpeed as u8,
                    // Subcommand payload
                    speed,
                    // max_power.to_u8(),
                    max_power,
                    profile,
                ]
            }
            StartSpeedForDegrees {
                degrees,
                speed,
                max_power,
                end_state,
                use_acc_profile,
                use_dec_profile,
            } => {
                let profile =
                    ((*use_acc_profile as u8) << 1) | (*use_dec_profile as u8);
                let speed = speed.to_le_bytes()[0];
                let max_power = max_power.to_le_bytes()[0];
                let degrees = degrees.to_le_bytes();
                let mut bytes = vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortOutputCommand as u8,
                    // Command
                    self.port_id,
                    0x11, // 0001 Execute immediately, 0001 Command feedback
                    PortOutputSubCommandValue::StartSpeedForDegrees as u8,
                ];
                // Subcommand payload
                bytes.extend_from_slice(&degrees);
                bytes.push(speed);
                // bytes.push(max_power.to_u8());
                bytes.push(max_power);
                bytes.push(end_state.to_u8());
                bytes.push(profile);

                bytes
            }
            GotoAbsolutePosition {
                abs_pos,
                speed,
                max_power,
                end_state,
                use_acc_profile,
                use_dec_profile,
            } => {
                let profile =
                    ((*use_acc_profile as u8) << 1) | (*use_dec_profile as u8);
                let speed = speed.to_le_bytes()[0];
                let max_power = max_power.to_le_bytes()[0];
                let abs_pos = abs_pos.to_le_bytes();
                dbg!(abs_pos);
                let mut bytes = vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortOutputCommand as u8,
                    // Command
                    self.port_id,
                    0x11, // 0001 Execute immediately, 0001 Command feedback
                    PortOutputSubCommandValue::StartSpeedForDegrees as u8,
                ];
                // Subcommand payload
                bytes.extend_from_slice(&abs_pos);
                bytes.push(speed);
                bytes.push(max_power);
                bytes.push(end_state.to_u8());
                bytes.push(profile);

                bytes
            }
            StartSpeedForTime {
                time,
                speed,
                max_power,
                end_state,
                use_acc_profile,
                use_dec_profile,
            } => {
                let profile =
                    ((*use_acc_profile as u8) << 1) | (*use_dec_profile as u8);
                let speed = speed.to_le_bytes()[0];
                let max_power = max_power.to_le_bytes()[0];
                let time = time.to_le_bytes();
                dbg!(time);
                let mut bytes = vec![
                    // Header
                    0, // len
                    0, // hub id - always set to 0
                    MessageType::PortOutputCommand as u8,
                    // Command
                    self.port_id,
                    0x11, // 0001 Execute immediately, 0001 Command feedback
                    PortOutputSubCommandValue::StartSpeedForDegrees as u8,
                ];
                // Subcommand payload
                bytes.extend_from_slice(&time);
                bytes.push(speed);
                // bytes.push(max_power.to_u8());
                bytes.push(max_power);
                bytes.push(end_state.to_u8());
                bytes.push(profile);

                bytes
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
        max_power: u8,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedNoPower {
        speed: i8,
        max_power: u8,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeed2 {
        speed1: i8,
        speed2: i8,
        max_power: u8,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForTime {
        time: i16,
        speed: i8,
        max_power: u8,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForTime2 {
        time: i16,
        speed_l: i8,
        speed_r: i8,
        max_power: u8,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForDegrees {
        degrees: i32,
        speed: i8,
        max_power: u8,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    StartSpeedForDegrees2 {
        degrees: i32,
        speed_l: i8,
        speed_r: i8,
        max_power: u8,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    GotoAbsolutePosition {
        abs_pos: i32,
        speed: i8,
        max_power: u8,
        end_state: EndState,
        use_acc_profile: bool,
        use_dec_profile: bool,
    },
    GotoAbsolutePosition2 {
        abs_pos1: i32,
        abs_pos2: i32,
        speed: i8,
        max_power: u8,
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
                // let max_power = Power::parse(&mut msg)?;
                let max_power = next!(msg);
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
            0x50 => {
                // WriteDirect(Byte[0],Byte[0 + n])
                let data = WriteDirectPayload::parse(&mut msg)?;
                WriteDirect(data)
            }
            0x51 => {
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
pub enum Speed {
    Cw(u8),
    Ccw(u8),
    Hold,
}

impl Speed {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let val = next_i8!(msg);
        Speed::from_i8(val)
    }

    pub fn to_u8(&self) -> u8 {
        use Speed::*;
        let integer: i8 = match self {
            Hold => 0,
            Cw(p) => *p as i8,
            Ccw(p) => -(*p as i8),
        };
        integer.to_le_bytes()[0]
    }

    pub fn from_i8(val: i8) -> Result<Self> {
        use Speed::*;
        match val {
            0 => Ok(Hold),
            p if (1..=100).contains(&p) => Ok(Cw(p as u8)),
            p if (-100..=-1).contains(&p) => Ok(Ccw((-p) as u8)),
            p => Err(Error::ParseError(format!(
                "Invalid value for Speed: {}",
                p
            ))),
        }
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
impl EndState {
    pub fn to_u8(&self) -> u8 {
        use EndState::*;
        let integer: u8 = match self {
            Float => 0,
            Hold => 126,
            Brake => 127,
        };
        integer.to_le_bytes()[0]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteDirectModeDataPayload {
    StartPower(Power),
    // StartPower2 has the "Encoded through WriteDirectModeData"-tag, but it also has a subcommand-id (0x02)
    // and is not listed among the WriteDirectModeData-commands. I think the tag is a doc error, so:
    // StartPower2{
    // power1: Power,
    // power2: Power
    // },
    // i32 as four bytes
    PresetEncoder(i32),
    TiltImpactPreset(i32),
    TiltConfigOrientation(Orientation),
    TiltConfigImpact {
        impact_threshold: i8,
        bump_holdoff: i8,
    },
    TiltFactoryCalibration(i8),
    SetHubColor(i8),
    SetHubRgb {
        red: u8,
        green: u8,
        blue: u8,
    },
    SetVisionSensorColor(i8),
}

impl WriteDirectModeDataPayload {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        use WriteDirectModeDataPayload::*;

        let mode = next!(msg);
        Ok(match mode {
            0x01 => {
                //Should be 0x00 according to docs? Seems to work, I may be misreading.
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
            0x07 => {
                // TiltFactoryCalibration(Orientation, CalibrationPassCode)  Passcode is 12 chars: Calib-Sensor
                // "Mode 7"
                let orientation = next_i8!(msg);
                // let passcode = next_i8!(msg);
                TiltFactoryCalibration(orientation)
            }
            0x08 => {
                // SetHubColor(ColorNo)
                let col = next_i8!(msg);
                SetHubColor(col)
            }
            0x09 => {
                // SetHubRgb(RedColor, GreenColor, BlueColor)
                let red = next!(msg);
                let green = next!(msg);
                let blue = next!(msg);
                SetHubRgb { red, green, blue }
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
            SetHubRgb { red, green, blue } => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    // Docs says to insert an 0x00 and then an extra 0x51 here, but works without it
                    crate::iodevice::modes::HubLed::RGB_O,
                    *red,
                    *green,
                    *blue,
                ]
            }
            SetHubColor(c) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::HubLed::COL_O,
                    *c as u8,
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
                    crate::iodevice::modes::InternalMotorTacho::POWER,
                    power,
                ]
            }
            PresetEncoder(position) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                let pos_bytes: [u8; 4] = position.to_le_bytes(); // i32 sent as 4 bytes
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::InternalMotorTacho::POS,
                    pos_bytes[0],
                    pos_bytes[1],
                    pos_bytes[2],
                    pos_bytes[3],
                ]
            }
            // Set the Tilt into TiltImpactCount mode (0x03) and change (preset) the value to PresetValue.
            TiltImpactPreset(preset_value) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                let val_bytes: [u8; 4] = preset_value.to_le_bytes(); // i32 sent as 4 bytes
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::InternalTilt::IMPCT,
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]
            }
            // Set the Tilt into TiltOrientation mode (0x05) and set the Orientation value to Orientation
            TiltConfigOrientation(orientation) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::InternalTilt::OR_CF,
                    *orientation as u8,
                ]
            }
            // Setup Tilt ImpactThreshold and BumpHoldoff by entering mode 6 and use the payload ImpactThreshold and BumpHoldoff.
            TiltConfigImpact {
                impact_threshold,
                bump_holdoff,
            } => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::InternalTilt::IM_CF,
                    *impact_threshold as u8,
                    *bump_holdoff as u8,
                ]
            }
            //  Sets the actual orientation in the montage automat.  0: XY (laying flat) 1: Z (standing long direction)
            TiltFactoryCalibration(orientation) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::InternalTilt::CALIB,
                    *orientation as u8,
                    b'C',
                    b'a',
                    b'l',
                    b'i',
                    b'b',
                    b'-',
                    b'S',
                    b'e',
                    b'n',
                    b's',
                    b'o',
                    b'r',
                ]
            }
            SetVisionSensorColor(c) => {
                let startup_and_completion =
                    meta.startup_info.serialise(&meta.completion_info);
                vec![
                    0,
                    0, // hub id
                    MessageType::PortOutputCommand as u8,
                    meta.port_id,
                    startup_and_completion,
                    0x51, // WriteDirect
                    crate::iodevice::modes::VisionSensor::COL_O,
                    *c as u8,
                ]
            } // _ => todo!(),
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
