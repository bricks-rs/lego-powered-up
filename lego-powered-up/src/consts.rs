// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Various constants defined by the specification, but translated into Rust
//! types

use num_derive::FromPrimitive;
use std::fmt::{self, Display};

/// ```text,ignore
/// @typedef DeviceType
/// @property {number} UNKNOWN 0
/// @property {number} SIMPLE_MEDIUM_LINEAR_MOTOR 1
/// @property {number} TRAIN_MOTOR 2
/// @property {number} LED_LIGHTS 8
/// @property {number} VOLTAGE 20
/// @property {number} CURRENT 21
/// @property {number} PIEZO_TONE 22
/// @property {number} RGB_LIGHT 23
/// @property {number} WEDO2_TILT 34
/// @property {number} WEDO2_DISTANCE 35
/// @property {number} COLOR_DISTANCE_SENSOR 37
/// @property {number} MEDIUM_LINEAR_MOTOR 38
/// @property {number} MOVE_HUB_MEDIUM_LINEAR_MOTOR 39
/// @property {number} BOOST_TILT 40
/// @property {number} DUPLO_TRAIN_BASE_MOTOR 41
/// @property {number} DUPLO_TRAIN_BASE_SPEAKER 42
/// @property {number} DUPLO_TRAIN_BASE_COLOR 43
/// @property {number} DUPLO_TRAIN_BASE_SPEEDOMETER 44
/// @property {number} CONTROL_PLUS_LARGE_MOTOR 46
/// @property {number} CONTROL_PLUS_XLARGE_MOTOR 47
/// @property {number} POWERED_UP_REMOTE_BUTTON 55
/// @property {number} RSSI 56
/// @property {number} CONTROL_PLUS_ACCELEROMETER 58
/// @property {number} CONTROL_PLUS_TILT 59
/// ```

// Added more IDs, some observed and some from
// https://github.com/nathankellenicki/node-poweredup/blob/master/src/consts.ts
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Default)]
pub enum IoTypeId {
    #[default]
    Unknown = 0x00,
    Motor = 0x01,
    SystemTrainMotor = 0x02,
    Button = 0x05,
    LedLight = 0x08,
    Voltage = 0x14,        //20
    Current = 0x15,        //21
    PiezoToneSound = 0x16, //22
    HubLed = 0x17,         //23

    ExternalTiltSensor = 0x22,      //34
    MotionSensor = 0x23,            //35
    VisionSensor = 0x25,            //37
    ExternalMotorTacho = 0x26,      //38
    InternalMotorTacho = 0x27,      //39
    InternalTilt = 0x28,            //40
    DuploTrainBaseMotor = 41,       //41
    DuploTrainBaseSpeaker = 42,     //42
    DuploTrainBaseColorSensor = 43, //43
    DuploTrainBaseSpeedometer = 44, //44
    TechnicLargeLinearMotor = 46,   //46   // Technic Control+
    TechnicXLargeLinearMotor = 47,  //47   // Technic Control+
    TechnicMediumAngularMotor = 48, //48   // Spike Prime
    TechnicLargeAngularMotor = 49,  //49   // Spike Prime

    TechnicHubGestSensor = 0x36,        //54
    RemoteButtons = 0x37,               //55
    Rssi = 0x38,                        //56
    TechnicHubAccelerometer = 0x39,     //57
    TechnicHubGyroSensor = 0x3a,        //58
    TechnicHubTiltSensor = 0x3b,        //59
    TechnicHubTemperatureSensor = 0x3c, //60
    TechnicColorSensor = 61,            //61    // Spike Prime
    TechnicDistanceSensor = 62,         //62    // Spike Prime
    TechnicForceSensor = 63,            //63    // Spike Prime
    Technic3x3ColorLightMatrix = 64,    //64    // Spike Essential
    TechnicSmallAngularMotor = 65,      //65    // Spike Essential
    UnknownMovehubDevice = 0x42, //66    // Unknown MoveHub device. Its 3 modes are named TRIGGER, CANVAS and VAR

    MarioAccelerometer = 71,            //71
    MarioBarcodeSensor = 73,            //73
    MarioPantsSensor = 74,              //74
    TechnicMediumAngularMotorGrey = 75, //75     // Mindstorms
    TechnicLargeAngularMotorGrey = 76,  //76     // Technic Control+
}

/// ```text,ignore
/// @typedef HubType
/// @property {number} UNKNOWN 0
/// @property {number} WEDO2_SMART_HUB 1
/// @property {number} MOVE_HUB 2
/// @property {number} POWERED_UP_HUB 3
/// @property {number} POWERED_UP_REMOTE 4
/// @property {number} DUPLO_TRAIN_HUB 5
/// @property {number} CONTROL_PLUS_HUB 6
/// @property {number} MARIO 7
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HubType {
    Unknown = 0,
    Wedo2SmartHub = 1,
    MoveHub = 2,
    Hub = 3,
    RemoteControl = 4,
    DuploTrainBase = 5,
    TechnicMediumHub = 6,
    Mario = 7,
}

impl Display for HubType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use HubType::*;
        match self {
            Unknown | MoveHub | Hub | Mario => write!(fmt, "{:?}", self),
            Wedo2SmartHub => write!(fmt, "Wedo 2 Smart Hub"),
            RemoteControl => write!(fmt, "Remote Control"),
            DuploTrainBase => write!(fmt, "Duplo Train Base"),
            TechnicMediumHub => write!(fmt, "Technic Medium Hub"),
        }
    }
}

/// ```text,ignore
/// @typedef Color
/// @property {number} BLACK 0
/// @property {number} PINK 1
/// @property {number} PURPLE 2
/// @property {number} BLUE 3
/// @property {number} LIGHT_BLUE 4
/// @property {number} CYAN 5
/// @property {number} GREEN 6
/// @property {number} YELLOW 7
/// @property {number} ORANGE 8
/// @property {number} RED 9
/// @property {number} WHITE 10
/// @property {number} NONE 255
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Pink = 1,
    Purple = 2,
    Blue = 3,
    LightBlue = 4,
    Cyan = 5,
    Green = 6,
    Yellow = 7,
    Orange = 8,
    Red = 9,
    White = 10,
    None = 255,
}

pub const LEGO_COLORS: [Color; 11] = [
    Color::Pink,
    Color::Purple,
    Color::Blue,
    Color::LightBlue,
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Orange,
    Color::Red,
    Color::White,
    Color::Black,
];

pub mod named_port {
    pub const A: u8 = 0x00;
    pub const B: u8 = 0x01;
    pub const C: u8 = 0x02;
    pub const D: u8 = 0x03;
    pub const MOVE_AB: u8 = 0x10;
}

// @typedef ButtonState
// @property {number} PRESSED 0
// @property {number} RELEASED 1
// @property {number} UP 2
// @property {number} DOWN 3
// @property {number} STOP 4

/// ```text,ignore
/// @typedef BrakingStyle
/// @property {number} HOLD 127
/// @property {number} BRAKE 128
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BrakingStyle {
    Float = 0,
    Hold = 126,
    Brake = 127,
}

/// ```text,ignore
/// @typedef DuploTrainBaseSound
/// @property {number} BRAKE 3
/// @property {number} STATION_DEPARTURE 5
/// @property {number} WATER_REFILL 7
/// @property {number} HORN 9
/// @property {number} STEAM 10
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DuploTrainBaseSound {
    Brake = 3,
    StationDeparture = 5,
    WaterRefill = 7,
    Horn = 9,
    Steam = 10,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum BLEManufacturerData {
    DuploTrainBaseId = 32,
    MoveHubId = 64,
    HubId = 65,
    RemoteControlId = 66,
    MarioId = 67,
    TechnicMediumHubId = 128,
}

pub mod bleservice {
    use lazy_static::lazy_static;
    use uuid::Uuid;

    pub const WEDO2_SMART_HUB_2: &str = "00004f0e-1212-efde-1523-785feabcd123";
    pub const WEDO2_SMART_HUB_3: &str = "2a19";
    pub const WEDO2_SMART_HUB_4: &str = "180f";
    pub const WEDO2_SMART_HUB_5: &str = "180a";
    lazy_static! {
        pub static ref WEDO2_SMART_HUB: Uuid =
            Uuid::parse_str("00001523-1212-efde-1523-785feabcd123").unwrap();
        pub static ref LPF2_HUB: Uuid =
            Uuid::parse_str("00001623-1212-efde-1623-785feabcd123").unwrap();
    }
}

pub mod blecharacteristic {
    use lazy_static::lazy_static;
    use uuid::Uuid;

    pub const WEDO2_BATTERY: &str = "2a19";
    pub const WEDO2_FIRMWARE_REVISION: &str = "2a26";
    pub const WEDO2_BUTTON: &str = "00001526-1212-efde-1523-785feabcd123"; // "1526"
    pub const WEDO2_PORT_TYPE: &str = "00001527-1212-efde-1523-785feabcd123"; // "1527" // Handles plugging and unplugging of devices on WeDo 2.0 Smart Hub
    pub const WEDO2_LOW_VOLTAGE_ALERT: &str =
        "00001528-1212-efde-1523-785feabcd123"; // "1528"
    pub const WEDO2_HIGH_CURRENT_ALERT: &str =
        "00001529-1212-efde-1523-785feabcd123"; // "1529"
    pub const WEDO2_LOW_SIGNAL_ALERT: &str =
        "0000152a-1212-efde-1523-785feabcd123"; // "152a",
    pub const WEDO2_DISCONNECT: &str = "0000152b-1212-efde-1523-785feabcd123"; // "152b"
    pub const WEDO2_SENSOR_VALUE: &str = "00001560-1212-efde-1523-785feabcd123"; // "1560"
    pub const WEDO2_VALUE_FORMAT: &str = "00001561-1212-efde-1523-785feabcd123"; // "1561"
    pub const WEDO2_PORT_TYPE_WRITE: &str =
        "00001563-1212-efde-1523-785feabcd123"; // "1563"
    pub const WEDO2_MOTOR_VALUE_WRITE: &str =
        "00001565-1212-efde-1523-785feabcd123"; // "1565"
    pub const WEDO2_NAME_ID: &str = "00001524-1212-efde-1523-785feabcd123"; // "1524"
    lazy_static! {
        pub static ref LPF2_ALL: Uuid =
            Uuid::parse_str("00001624-1212-efde-1623-785feabcd123").unwrap();
    }
}

/// ```text,ignore
/// @typedef MessageType
/// @property {number} HUB_PROPERTIES 0x01
/// @property {number} HUB_ACTIONS 0x02
/// @property {number} HUB_ALERTS 0x03
/// @property {number} HUB_ATTACHED_IO 0x04
/// @property {number} GENERIC_ERROR_MESSAGES 0x05
/// @property {number} HW_NETWORK_COMMANDS 0x08
/// @property {number} FW_UPDATE_GO_INTO_BOOT_MODE 0x10
/// @property {number} FW_UPDATE_LOCK_MEMORY 0x11
/// @property {number} FW_UPDATE_LOCK_STATUS_REQUEST 0x12
/// @property {number} FW_LOCK_STATUS 0x13
/// @property {number} PORT_INFORMATION_REQUEST 0x21
/// @property {number} PORT_MODE_INFORMATION_REQUEST 0x22
/// @property {number} PORT_INPUT_FORMAT_SETUP_SINGLE 0x41
/// @property {number} PORT_INPUT_FORMAT_SETUP_COMBINEDMODE 0x42
/// @property {number} PORT_INFORMATION 0x43
/// @property {number} PORT_MODE_INFORMATION 0x44
/// @property {number} PORT_VALUE_SINGLE 0x45
/// @property {number} PORT_VALUE_COMBINEDMODE 0x46
/// @property {number} PORT_INPUT_FORMAT_SINGLE 0x47
/// @property {number} PORT_INPUT_FORMAT_COMBINEDMODE 0x48
/// @property {number} VIRTUAL_PORT_SETUP 0x61
/// @property {number} PORT_OUTPUT_COMMAND 0x81
/// @property {number} PORT_OUTPUT_COMMAND_FEEDBACK 0x82
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#message-types>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, Hash)]
pub enum MessageType {
    HubProperties = 0x01,
    HubActions = 0x02,
    HubAlerts = 0x03,
    HubAttachedIo = 0x04,
    GenericErrorMessages = 0x05,
    HwNetworkCommands = 0x08,
    FwUpdateGoIntoBootMode = 0x10,
    FwUpdateLockMemory = 0x11,
    FwUpdateLockStatusRequest = 0x12,
    FwLockStatus = 0x13,
    PortInformationRequest = 0x21,
    PortModeInformationRequest = 0x22,
    PortInputFormatSetupSingle = 0x41,
    PortInputFormatSetupCombined = 0x42,
    PortInformation = 0x43,
    PortModeInformation = 0x44,
    PortValueSingle = 0x45,
    PortValueCombined = 0x46,
    PortInputFormatSingle = 0x47,
    PortInputFormatCombined = 0x48,
    VirtualPortSetup = 0x61,
    PortOutputCommand = 0x81,
    PortOutputCommandFeedback = 0x82,
}

/// ```text,ignore
/// @typedef HubPropertyReference
/// @param {number} ADVERTISING_NAME 0x01
/// @param {number} BUTTON 0x02
/// @param {number} FW_VERSION 0x03
/// @param {number} HW_VERSION 0x04
/// @param {number} RSSI 0x05
/// @param {number} BATTERY_VOLTAGE 0x06
/// @param {number} BATTERY_TYPE 0x07
/// @param {number} MANUFACTURER_NAME 0x08
/// @param {number} RADIO_FIRMWARE_VERSION 0x09
/// @param {number} LEGO_WIRELESS_PROTOCOL_VERSION 0x0A
/// @param {number} SYSTEM_TYPE_ID 0x0B
/// @param {number} HW_NETWORK_ID 0x0C
/// @param {number} PRIMARY_MAC_ADDRESS 0x0D
/// @param {number} SECONDARY_MAC_ADDRESS 0x0E
/// @param {number} HARDWARE_NETWORK_FAMILY 0x0F
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#hub-property-reference>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum HubPropertyRef {
    AdvertisingName = 0x01,
    Button = 0x02,
    FwVersion = 0x03,
    HwVersion = 0x04,
    Rssi = 0x05,
    BatteryVoltage = 0x06,
    BatteryType = 0x07,
    ManufacturerName = 0x08,
    RadioFirmwareVersion = 0x09,
    LegoWirelessProtocolVersion = 0x0A,
    SystemTypeId = 0x0B,
    HwNetworkId = 0x0C,
    PrimaryMacAddress = 0x0D,
    SecondaryMacAddress = 0x0E,
    HardwareNetworkFamily = 0x0F,
}

/// ```text,ignore
/// @typedef HubPropertyOperation
/// @param {number} SET_DOWNSTREAM 0x01
/// @param {number} ENABLE_UPDATES_DOWNSTREAM 0x02
/// @param {number} DISABLE_UPDATES_DOWNSTREAM 0x03
/// @param {number} RESET_DOWNSTREAM 0x04
/// @param {number} REQUEST_UPDATE_DOWNSTREAM 0x05
/// @param {number} UPDATE_UPSTREAM 0x06
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#hub-property-reference>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum HubPropertyOperation {
    SetDownstream = 0x01,
    EnableUpdatesDownstream = 0x02,
    DisableUpdatesDownstream = 0x03,
    ResetDownstream = 0x04,
    RequestUpdateDownstream = 0x05,
    UpdateUpstream = 0x06,
}

/// ```text,ignore
/// @typedef HubPropertyPayload
/// @param {number} ADVERTISING_NAME 0x01
/// @param {number} BUTTON_STATE 0x02
/// @param {number} FW_VERSION 0x03
/// @param {number} HW_VERSION 0x04
/// @param {number} RSSI 0x05
/// @param {number} BATTERY_VOLTAGE 0x06
/// @param {number} BATTERY_TYPE 0x07
/// @param {number} MANUFACTURER_NAME 0x08
/// @param {number} RADIO_FIRMWARE_VERSION 0x09
/// @param {number} LWP_PROTOCOL_VERSION 0x0A
/// @param {number} SYSTEM_TYPE_ID 0x0B
/// @param {number} HW_NETWORK_ID 0x0C
/// @param {number} PRIMARY_MAC_ADDRESS 0x0D
/// @param {number} SECONDARY_MAC_ADDRESS 0x0E
/// @param {number} HW_NETWORK_FAMILY 0x0F
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#hub-property-reference>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum HubPropertyPayload {
    AdvertisingName = 0x01,
    ButtonState = 0x02,
    FwVersion = 0x03,
    HwVersion = 0x04,
    Rssi = 0x05,
    BatteryVoltage = 0x06,
    BatteryType = 0x07,
    ManufacturerName = 0x08,
    RadioFirmwareVersion = 0x09,
    LwpProtocolVersion = 0x0A,
    SystemTypeId = 0x0B,
    HwNetworkId = 0x0C,
    PrimaryMacAddress = 0x0D,
    SecondaryMacAddress = 0x0E,
    HwNetworkFamily = 0x0F,
}

/// ```text,ignore
/// @typedef ActionType
/// @param {number} SWITCH_OFF_HUB 0x01
/// @param {number} DISCONNECT 0x02
/// @param {number} VCC_PORT_CONTROL_ON 0x03
/// @param {number} VCC_PORT_CONTROL_OFF 0x04
/// @param {number} ACTIVATE_BUSY_INDICATION 0x05
/// @param {number} RESET_BUSY_INDICATION 0x06
/// @param {number} SHUTDOWN 0x2F
/// @param {number} HUB_WILL_SWITCH_OFF 0x30
/// @param {number} HUB_WILL_DISCONNECT 0x31
/// @param {number} HUB_WILL_GO_INTO_BOOT_MODE 0x32
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#action-types>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionType {
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

/// ```text,ignore
/// @typedef AlertPayload
/// @param {number} STATUS_OK 0x00
/// @param {number} ALERT 0xFF
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#alert-payload>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AlertPayload {
    StatusOk = 0x00,
    Alert = 0xFF,
}

/// ```text,ignore
/// @typedef Event
/// @param {number} DETACHED_IO 0x00
/// @param {number} ATTACHED_IO 0x01
/// @param {number} ATTACHED_VIRTUAL_IO 0x02
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#event>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum Event {
    DetachedIo = 0x00,
    AttachedIo = 0x01,
    AttachedVirtualIo = 0x02,
}

/// ```text,ignore
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
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum HwNetworkCommandType {
    ConnectionRequest = 0x02,
    FamilyRequest = 0x03,
    FamilySet = 0x04,
    JoinDenied = 0x05,
    GetFamily = 0x06,
    Family = 0x07,
    GetSubfamily = 0x08,
    Subfamily = 0x09,
    SubfamilySet = 0x0A,
    GetExtendedFamily = 0x0B,
    ExtendedFamily = 0x0C,
    ExtendedFamilySet = 0x0D,
    ResetLongPressTiming = 0x0E,
}

/// ```text,ignore
/// @typedef PortInputFormatSetupSubCommand
/// @param {number} SET_MODEANDDATASET_COMBINATIONS 0x01
/// @param {number} LOCK_LPF2_DEVICE_FOR_SETUP 0x02
/// @param {number} UNLOCKANDSTARTWITHMULTIUPDATEENABLED 0x03
/// @param {number} UNLOCKANDSTARTWITHMULTIUPDATEDISABLED 0x04
/// @param {number} NOT_USED 0x05
/// @param {number} RESET_SENSOR 0x06
/// @description <https://lego.github.io/lego-ble-wireless-protocol-docs/index.html#port-input-format-setup-sub-commands>
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum PortInputFormatSetupSubCommand {
    SetModeanddatasetCombinations = 0x01,
    LockLpf2DeviceForSetup = 0x02,
    UnlockAndStartMultiEnabled = 0x03,
    UnlockAndStartMultiDisabled = 0x04,
    NotUsed = 0x05,
    ResetSensor = 0x06,
}

/// ```text,ignore
/// @typedef MarioPantsType
/// @param {number} NONE 0x00
/// @param {number} PROPELLER 0x06
/// @param {number} CAT 0x11
/// @param {number} FIRE 0x12
/// @param {number} NORMAL 0x21
/// @param {number} BUILDER 0x22
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MarioPantsType {
    None = 0x00,
    Propeller = 0x06,
    Cat = 0x11,
    Fire = 0x12,
    Normal = 0x21,
    Builder = 0x22,
}

/// ```text,ignore
/// @typedef MarioColor
/// @param {number} WHITE 0x1300
/// @param {number} RED 0x1500
/// @param {number} BLUE 0x1700
/// @param {number} YELLOW 0x1800
/// @param {number} BLACK 0x1a00
/// @param {number} GREEN 0x2500
/// @param {number} BROWN 0x6a00
/// @param {number} CYAN 0x4201
/// ```
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MarioColor {
    White = 0x1300,
    Red = 0x1500,
    Blue = 0x1700,
    Yellow = 0x1800,
    Black = 0x1a00,
    Green = 0x2500,
    Brown = 0x6a00,
    Cyan = 0x4201,
}

pub enum PortOutputSubCommandValue {
    StartPower2 = 0x02,
    SetAccTime = 0x05,
    SetDecTime = 0x06,
    StartSpeed = 0x07,
    StartSpeed2 = 0x08,
    StartSpeedForTime = 0x09,
    StartSpeedForTime2 = 0x0a,
    StartSpeedForDegrees = 0x0b,
    StartSpeedForDegrees2 = 0x0c,
    GotoAbsolutePosition = 0x0d,
    GotoAbsolutePosition2 = 0x0e,
    PresetEncoder2 = 0x14,
}

pub enum InputSetupCombinedSubcommandValue {
    SetModeanddatasetCombinations = 0x01,
    LockLpf2DeviceForSetup = 0x02,
    UnlockAndStartMultiEnabled = 0x03,
    UnlockAndStartMultiDisabled = 0x04,
    NotUsed = 0x05,
    ResetSensor = 0x06,
}

pub enum MotorSensorMode {
    // Valid combinations: 1+2+3  (Speed, Pos, Apos)
    Power = 0x0,
    Speed = 0x1,
    Pos = 0x2,
    APos = 0x3,
    // Load = 0x4,
    Calib = 0x4,
    Stats = 0x5,
}

pub enum VisionSensorMode {
    // Valid combinations: 0+1+2+3+6	Color, Prox, Count, Reflt, Rgb I
    Color = 0x0,
    Prox = 0x1,
    Count = 0x2,
    Reflt = 0x3,
    Ambi = 0x4,
    ColO = 0x5,
    RgbI = 0x6,
    IrTx = 0x7,
    Spec1 = 0x8,
    Debug = 0x9,
    Calib = 0xa,
}

// pub struct PortCapabilities(u8);
// impl PortCapabilities {
//     pub const LOGICAL_SYNCHRONIZABLE: u8 = 0b1000;
//     pub const LOGICAL_COMBINABLE: u8 = 0b0100;
//     pub const INPUT: u8 = 0b0010;
//     pub const OUTPUT: u8 = 0b0001;
// }
