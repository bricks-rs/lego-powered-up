// Representation of an IoDevice
//      This overlaps with previous definitions spread out through the lib, 
//      mostly in notifications and consts modules. Doing this to make organization
//      clearer for myself to start. Perhaps we'll use it if it helps. 

use crate::consts::*;

#[derive(Debug, Default)]
pub struct IoDevice {
    kind: IoTypeId,
    port: u8,
    capabilities: Vec<Capability>,
    modes: Vec<PortMode>,
    valid_combos: Vec<ModeCombo>,
}

#[derive(Debug, Default)]
pub struct PortMode {
    is_input: bool,             // true = inputmode, false = outputmode
    name: [char; 11],           // Transmitted from hub as [u8; 11]
    raw: (f32, f32),            // (min, max) The range for the raw (transmitted) signal, remember other ranges are used for scaling the value.
    pct: (f32, f32),            // (min, max) % scaling. Ex: RAW == 0-200 PCT == 0-100 => 100 RAW == 50%
    si_min: f32,                // (min, max) SI-unit scaling (probably?)
    symbol: [char; 5],          // Transmitted from hub as [u8; 5]
    input_mapping: Mapping,     // Cf. info below. Can more than 1 mapping be enabled?
    output_mapping: Mapping,
    motor_bias: u8,             // 0..100
    sensor_cabability: [u8; 6], // Sensor capabilities as bits, meaning not available.
    value_format: ValueFormat,
}

#[derive(Debug, Default)]
pub struct ModeCombo {
    modes: Vec<PortMode>
}
#[derive(Debug, Default)]
pub struct ValueFormat {
    dataset_count: u8,
    dataset_type: DatasetType,
    total_figures: u8,
    decimals: u8
}

// connected_io: Default::default(),

impl IoDevice {
    pub fn new() -> Self {
        Self {
            kind: Default::default(),
            port: Default::default(),
            capabilities: Default::default(),
            valid_combos: Default::default(),
            modes: Default::default()
        }
    }
}


#[derive(Debug, Default)]
pub enum DatasetType {
    // Transmitted as u8, upper 6 bits not used
    #[default]
    Unknown = 255,
    Bits8 = 0b00,
    Bits16 = 0b01,
    Bits32 = 0b10,
    Float = 0b11,
}

#[derive(Debug, Default)]
pub enum Capability {
    // Transmitted as u8, upper nibble not used
    #[default]
    Unknown,
    LogicalSynchronizable = 0b1000,
    LogicalCombinable = 0b0100,
    ProvideData = 0b0010,           // Input (seen from Hub)
    TakeData = 0b0001,              // Output (seen from Hub)
}

#[derive(Debug, Default)]
pub enum Mapping {
    #[default]
    Unknown,
    SupportsNull = 0b1000_0000,
    SupportsFunctional = 0b0100_0000,
    // bit 5 not used
    Absolute = 0b0001_0000,             // ABS (Absolute [min..max])
    Relative = 0b0000_1000,             // REL (Relative [-1..1])
    Discrete = 0b0000_0100              // DIS (Discrete [0, 1, 2, 3])
    // bit 1 not used
    // bit 0 not used
}


// Input mapping info from docs:
// The roles are: The host of the sensor (even a simple and dumb black box)
// can then decide, what to do with the sensor without any setup (default
// mode 0 (zero). Using the LSB first (highest priority).