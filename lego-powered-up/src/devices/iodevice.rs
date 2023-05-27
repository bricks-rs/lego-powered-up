// Representation of an IoDevice
//      This overlaps with previous definitions spread out through the lib, 
//      mostly in notifications and consts modules. Doing this to make organization
//      clearer for myself to start. Perhaps we'll use it if it helps. 

use crate::consts::*;
use std::collections::HashMap;
type ModeId = u8;
#[derive(Debug, Default)]
pub struct IoDevice {
    pub kind: IoTypeId,
    pub port: u8,
    capabilities: Vec<Capability>,
    pub mode_count: u8,
    modes: HashMap<ModeId, PortMode>,
    valid_combos: Vec<ModeCombo>,
}

#[derive(Debug, Default)]
pub struct PortMode {
    is_input: bool,             // true = inputmode, false = outputmode
    name: String,           // Transmitted from hub as [u8; 11]
    raw: (f32, f32),            // (min, max) The range for the raw (transmitted) signal, remember other ranges are used for scaling the value.
    pct: (f32, f32),            // (min, max) % scaling. Ex: RAW == 0-200 PCT == 0-100 => 100 RAW == 50%
    si: (f32, f32),                // (min, max) SI-unit scaling (probably?)
    symbol: String,          // Transmitted from hub as [u8; 5]
    input_mapping: Mapping,     // Cf. info below. Can more than 1 mapping be enabled?
    output_mapping: Mapping,
    motor_bias: u8,             // 0..100
    sensor_cabability: [u8; 6], // Sensor capabilities as bits, docs unclear on meaning.
    value_format: ValueFormat,
}
impl PortMode {
    pub fn new(is_input: bool) -> Self {
        Self {
            is_input,                   
            name: Default::default(),   
            raw: Default::default(),            
            pct: Default::default(),            
            si: Default::default(),                
            symbol: Default::default(),          
            input_mapping: Default::default(),     
            output_mapping: Default::default(),
            motor_bias: Default::default(),             
            sensor_cabability: Default::default(), 
            value_format: Default::default(),
        }
    }
    pub fn set_name(&self, chars: Vec<u8>) -> () {

    }
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
    pub fn new(kind: IoTypeId, port: u8) -> Self {
        Self {
            kind,
            port,
            mode_count: Default::default(),
            capabilities: Default::default(),
            valid_combos: Default::default(),
            modes: Default::default()
        }
    }
    pub fn set_mode_count(&mut self, mode_count: u8) -> () {
        self.mode_count = mode_count;
    }
    pub fn set_modes(&mut self, input_modes: u16, output_modes: u16) -> () {
        let mut r: HashMap<ModeId, PortMode> = HashMap::new();
        for mode in (0..15) {
            if (input_modes >> mode as u16) & 1 == 1 {
                r.insert(mode as u8, PortMode::new(true));
            }    
        }
        for mode in (0..15) {
            if (output_modes >> mode as u16) & 1 == 1 {
                r.insert(mode as u8, PortMode::new(false));
            }    
        }
        self.modes = r;
    }
    pub fn get_modes(&self) -> &HashMap<ModeId, PortMode> {
        &self.modes
    } 
    
    pub fn set_capabilities(&mut self, capabilities: u8) -> () {
        let mut r: Vec<Capability> = Vec::new();
        if (capabilities >> 3) & 1 == 1 {r.push(Capability::LogicalSynchronizable)}
        if (capabilities >> 2) & 1 == 1 {r.push(Capability::LogicalCombinable)}
        if (capabilities >> 1) & 1 == 1 {r.push(Capability::ProvideData)}
        if (capabilities >> 0) & 1 == 1 {r.push(Capability::AcceptData)}
        self.capabilities = r;
    }

    pub fn set_mode_name(&mut self, mode_id: u8, chars: Vec<u8>) -> () {
        let name = String::from_utf8(chars).expect("Found invalid UTF-8");
        let mut mode = self.modes.get_mut(&mode_id).unwrap();
        mode.name = name;
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
    None,
    LogicalSynchronizable = 0b1000,
    LogicalCombinable = 0b0100,
    ProvideData = 0b0010,           // Input (seen from Hub)
    AcceptData = 0b0001,              // Output (seen from Hub)
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