use crate::consts::*;
use anyhow::{bail, Context, Result};
use lpu_macros::Parse;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;

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

macro_rules! next_i32 {
    ($iter:ident) => {
        i32::from_le_bytes([
            next!($iter),
            next!($iter),
            next!($iter),
            next!($iter),
        ])
    };
}

macro_rules! next_u32 {
    ($iter:ident) => {
        u32::from_le_bytes([
            next!($iter),
            next!($iter),
            next!($iter),
            next!($iter),
        ])
    };
}

macro_rules! next_f32 {
    ($iter:ident) => {
        f32::from_le_bytes([
            next!($iter),
            next!($iter),
            next!($iter),
            next!($iter),
        ])
    };
}

macro_rules! next_u16 {
    ($iter:ident) => {
        u16::from_le_bytes([next!($iter), next!($iter)])
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
    HubActions(HubAction),
    HubAlerts(AlertType),
    HubAttachedIo(AttachedIo),
    GenericErrorMessages(ErrorCode),
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
                let error = ErrorCode::parse(&mut msg_iter)?;
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
            _ => todo!(),
        })
    }

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
#[derive(Copy, Clone, Debug, FromPrimitive, Parse)]
pub enum HubBatteryType {
    Normal = 0x00,
    Rechargeable = 0x01,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive, Parse)]
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

#[derive(Copy, Clone, Debug)]
pub struct AttachedIo {
    port: u8,
    event: IoAttachEvent,
}

impl AttachedIo {
    pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) -> Result<Self> {
        let port = next!(msg);
        let event = IoAttachEvent::parse(&mut msg)?;
        Ok(Self { port, event })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum IoAttachEvent {
    DetachedIo { io_type_id: IoTypeId },
    AttachedIo { hw_rev: i32, fw_rev: i32 },
    AttachedVirtualIo { port_a: u8, port_b: u8 },
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
                let hw_rev = next_i32!(msg);
                let fw_rev = next_i32!(msg);
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

/**
 * @typedef HWNetWorkCommandType
 * @param {number} CONNECTION_REQUEST 0x02
 * @param {number} FAMILY_REQUEST 0x03
 * @param {number} FAMILY_SET 0x04
 * @param {number} JOIN_DENIED 0x05
 * @param {number} GET_FAMILY 0x06
 * @param {number} FAMILY 0x07
 * @param {number} GET_SUBFAMILY 0x08
 * @param {number} SUBFAMILY 0x09
 * @param {number} SUBFAMILY_SET 0x0A
 * @param {number} GET_EXTENDED_FAMILY 0x0B
 * @param {number} EXTENDED_FAMILY 0x0C
 * @param {number} EXTENDED_FAMILY_SET 0x0D
 * @param {number} RESET_LONG_PRESS_TIMING 0x0E
 * @description https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#h-w-network-command-type
 */
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
    port_id: u8,
    mode: u8,
    delta: u32,
    notification_enabled: bool,
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
            b => bail!("Invalid notification enabled state {:x}", b),
        };
        Ok(Self {
            port_id,
            mode,
            delta,
            notification_enabled,
        })
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

#[derive(Clone, Debug, PartialEq)]
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
        Ok(match mode {
            1 => {
                // Mode info
                let capabilities = PortCapabilities(next!(msg));
                let mode_count = next!(msg);
                let input_modes = next_u16!(msg);
                let output_modes = next_u16!(msg);
                ModeInfo {
                    capabilities,
                    mode_count,
                    input_modes,
                    output_modes,
                }
            }
            2 => {
                // possible mode combinations
                let combinations = msg.cloned().collect();
                PossibleModeCombinations(combinations)
            }
            m => bail!("Invalid port information type {}", m),
        })
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
            t => bail!("Invalid information type {}", t),
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
    pub const SUPPORTS_NULL: u8 = 0b1000_000;
    pub const SUPPORTS_FUNCTIONAL2: u8 = 0b0100_000;
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
            0 => false,
            1 => true,
            v => bail!("Invalid notification enabled status {}", v),
        };
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
        Ok(match next!(msg) {
            0 => {
                // Disconnected
                let port_id = next!(msg);
                Disconnect { port_id }
            }
            1 => {
                // Connected
                let port_a = next!(msg);
                let port_b = next!(msg);
                Connect { port_a, port_b }
            }
            c => bail!("Invalid virtual port subcommand {}", c),
        })
    }
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
