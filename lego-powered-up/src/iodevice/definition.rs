//! Models the information available about a device from
//! the AttachedIo, PortInformation and PortModeInformation
//! message types.

use std::collections::BTreeMap;
use std::fmt;

use crate::notifications::*;
use crate::IoTypeId;

type ModeId = u8;

#[derive(Debug, Default, Clone)]
pub struct Definition {
    kind: IoTypeId,
    port: u8,
    capabilities: Vec<Capability>,
    mode_count: u8,
    modes: std::collections::BTreeMap<ModeId, PortMode>,
    valid_combos: Vec<Vec<u8>>,
}
impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:#?} on port {} ({:#x}) with {} modes: {:#?}",
            self.kind,
            self.port,
            self.port,
            self.mode_count,
            self.modes
                .values()
                .map(|mode| &mode.name[..])
                .collect::<Vec<_>>()
        )
    }
}

impl Definition {
    pub fn kind(&self) -> &IoTypeId {
        &self.kind
    }
    pub fn port(&self) -> u8 {
        self.port
    }
    pub fn capabilities(&self) -> &Vec<Capability> {
        &self.capabilities
    }
    pub fn mode_count(&self) -> &u8 {
        &self.mode_count
    }
    pub fn modes(&self) -> &BTreeMap<ModeId, PortMode> {
        &self.modes
    }
    pub fn valid_combos(&self) -> &Vec<Vec<u8>> {
        &self.valid_combos
    }

    pub fn new(kind: IoTypeId, port: u8) -> Self {
        Self {
            kind,
            port,
            mode_count: Default::default(),
            capabilities: Default::default(),
            valid_combos: Default::default(),
            modes: Default::default(),
        }
    }
    pub fn set_mode_count(&mut self, mode_count: u8) {
        self.mode_count = mode_count;
    }
    pub fn set_modes(&mut self, input_modes: u16, output_modes: u16) {
        let mut r: BTreeMap<ModeId, PortMode> = BTreeMap::new();
        for mode in 0..15 {
            if (input_modes >> mode as u16) & 1 == 1 {
                r.insert(mode as u8, PortMode::new(ModeKind::Sensor));
            }
        }
        for mode in 0..15 {
            if (output_modes >> mode as u16) & 1 == 1 {
                r.insert(mode as u8, PortMode::new(ModeKind::Output));
            }
        }

        // Add hidden modes
        while r.len() < self.mode_count as usize {
            let mut empty_key = 0;
            while r.contains_key(&empty_key) {
                empty_key += 1
            }
            r.insert(empty_key, PortMode::new(ModeKind::Hidden));
        }
        self.modes = r;
    }
    // pub fn get_modes(&self) -> &BTreeMap<ModeId, PortMode> {
    //     &self.modes
    // }

    pub fn set_capabilities(&mut self, capabilities: u8) {
        let mut r: Vec<Capability> = Vec::new();
        if (capabilities >> 3) & 1 == 1 {
            r.push(Capability::LogicalSynchronizable)
        }
        if (capabilities >> 2) & 1 == 1 {
            r.push(Capability::LogicalCombinable)
        }
        if (capabilities >> 1) & 1 == 1 {
            r.push(Capability::ProvideData)
        }
        if capabilities & 1 == 1 {
            r.push(Capability::AcceptData)
        }
        self.capabilities = r;
    }

    pub fn set_valid_combos(&mut self, valid: Vec<u8>) {
        for combo in valid {
            let mut v: Vec<u8> = Vec::new();
            for mode in 0..7 {
                if (combo >> mode as u8) & 1 == 1 {
                    v.push(mode as u8);
                }
            }
            self.valid_combos.push(v);
        }
        self.valid_combos.pop(); // Last one is empty end-marker
    }

    pub fn set_mode_name(&mut self, mode_id: u8, chars_as_bytes: Vec<u8>) {
        // let mut truncated = vec![chars_as_bytes.into_iter)]; // iter with closure..?E
        let mut truncated: Vec<u8> = Vec::new();
        for c in chars_as_bytes {
            if c == 0 {
                break;
            } else {
                truncated.push(c)
            }
        }

        let name = String::from_utf8(truncated).expect("Found invalid UTF-8");
        let mode = self.modes.get_mut(&mode_id);
        match mode {
            Some(m) => m.name = name,
            None => {
                error!(
                    "Found name without matching mode. Port:{} Mode:{} Name:{}",
                    self.port, &mode_id, &name
                );
                println!(
                    "Found name without matching mode. Port:{} Mode:{} Name:{}",
                    self.port, &mode_id, &name
                );

                // Some devices have modes that  count towards mode_count but are not listed in available modes.
                // For example the TecnhicLargeLinearMotor has the "hidden" modes CALIB and STATS in addition to
                // normal modes POWER, SPEED, POS and APOS. The Vision Sensor has a few as well. They might be
                // useful for something, so we'll get their info as well and list them with ModeKind::Hidden.
            }
        }
    }
    pub fn set_mode_raw(&mut self, mode_id: u8, min: f32, max: f32) {
        // let mut mode =
        self.modes.get_mut(&mode_id).unwrap().raw = (min, max);
    }
    pub fn set_mode_pct(&mut self, mode_id: u8, min: f32, max: f32) {
        // let mut mode =
        self.modes.get_mut(&mode_id).unwrap().pct = (min, max);
    }
    pub fn set_mode_si(&mut self, mode_id: u8, min: f32, max: f32) {
        // let mut mode =
        self.modes.get_mut(&mode_id).unwrap().si = (min, max);
    }
    pub fn set_mode_symbol(&mut self, mode_id: u8, chars_as_bytes: Vec<u8>) {
        let mut truncated: Vec<u8> = Vec::new();
        for c in chars_as_bytes {
            if c == 0 {
                break;
            } else {
                truncated.push(c)
            }
        }
        let symbol = String::from_utf8(truncated).expect("Found invalid UTF-8");
        let mode = self.modes.get_mut(&mode_id);
        match mode {
            Some(m) => m.symbol = symbol,
            None => {
                error!("Found symbol without matching mode. Port:{} Mode:{} Symbol:{}", self.port, &mode_id, &symbol);
                println!("Found symbol without matching mode. Port:{} Mode:{} Symbol:{}", self.port, &mode_id, &symbol);
            }
        }
    }

    // Input mapping info from docs:
    // The roles are: The host of the sensor (even a simple and dumb black box)
    // can then decide, what to do with the sensor without any setup (default
    // mode 0 (zero). Using the LSB first (highest priority).
    pub fn set_mode_mapping(
        &mut self,
        mode_id: u8,
        input: MappingValue,
        output: MappingValue,
    ) {
        let mode = self.modes.get_mut(&mode_id).unwrap();
        let mut r: Vec<Mapping> = Vec::new();
        if (input.0 >> 7) & 1 == 1 {
            r.push(Mapping::SupportsNull)
        }
        if (input.0 >> 6) & 1 == 1 {
            r.push(Mapping::SupportsFunctional)
        }
        // if (input.0 >> 5) & 1 == 1 {}    // Not used
        if (input.0 >> 4) & 1 == 1 {
            r.push(Mapping::Absolute)
        }
        if (input.0 >> 3) & 1 == 1 {
            r.push(Mapping::Relative)
        }
        if (input.0 >> 2) & 1 == 1 {
            r.push(Mapping::Discrete)
        }
        // if (input.0 >> 1) & 1 == 1 {}    // Not used
        // if (input.0 >> 0) & 1 == 1 {}    // Not used
        mode.input_mapping = r;

        let mut r: Vec<Mapping> = Vec::new();
        if (output.0 >> 7) & 1 == 1 {
            r.push(Mapping::SupportsNull)
        }
        if (output.0 >> 6) & 1 == 1 {
            r.push(Mapping::SupportsFunctional)
        }
        // if (output.0 >> 5) & 1 == 1 {}   // Not used
        if (output.0 >> 4) & 1 == 1 {
            r.push(Mapping::Absolute)
        }
        if (output.0 >> 3) & 1 == 1 {
            r.push(Mapping::Relative)
        }
        if (output.0 >> 2) & 1 == 1 {
            r.push(Mapping::Discrete)
        }
        // if (output.0 >> 1) & 1 == 1 {}   // Not used
        // if (output.0 >> 0) & 1 == 1 {}   // Not used
        mode.output_mapping = r;
    }
    pub fn set_mode_valueformat(
        &mut self,
        mode_id: u8,
        format: ValueFormatType,
    ) {
        self.modes.get_mut(&mode_id).unwrap().value_format = format;
    }

    pub fn set_mode_motor_bias(&mut self, mode_id: u8, bias: u8) {
        // let mut mode =
        self.modes.get_mut(&mode_id).unwrap().motor_bias = bias;
    }
}

#[derive(Debug, Default, Clone)]
pub struct PortMode {
    pub kind: ModeKind,
    pub name: String,    // Transmitted from hub as [u8; 11]
    pub raw: (f32, f32), // (min, max) The range for the raw (transmitted) signal, remember other ranges are used for scaling the value.
    pub pct: (f32, f32), // (min, max) % scaling. Ex: RAW == 0-200 PCT == 0-100 => 100 RAW == 50%
    pub si: (f32, f32),  // (min, max) SI-unit scaling (probably?)
    pub symbol: String,  // Transmitted from hub as [u8; 5]
    pub input_mapping: Vec<Mapping>, // Cf. info below. Can more than 1 mapping be enabled? Yes.
    pub output_mapping: Vec<Mapping>,
    pub motor_bias: u8, // 0..100
    // pub sensor_cabability: [u8; 6],  // Sensor capabilities as bits. No help from docs how to interpret, just ignore it for now.
    pub value_format: ValueFormatType,
}
impl fmt::Display for PortMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<10?} {:<10} {:<10} {:}",
            self.kind, self.name, self.symbol, self.value_format
        )
    }
}

impl PortMode {
    pub fn new(mode_kind: ModeKind) -> Self {
        Self {
            kind: mode_kind,
            name: Default::default(),
            raw: Default::default(),
            pct: Default::default(),
            si: Default::default(),
            symbol: Default::default(),
            input_mapping: Default::default(),
            output_mapping: Default::default(),
            motor_bias: Default::default(),
            // sensor_cabability: Default::default(),
            value_format: Default::default(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ModeKind {
    #[default]
    Unknown,
    Sensor,
    Output,
    Hidden,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Capability {
    // Transmitted as u8, upper nibble not used
    #[default]
    None,
    LogicalSynchronizable = 0b1000,
    LogicalCombinable = 0b0100,
    ProvideData = 0b0010, // Input (seen from Hub)
    AcceptData = 0b0001,  // Output (seen from Hub)
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Mapping {
    #[default]
    Unknown,
    SupportsNull = 0b1000_0000,
    SupportsFunctional = 0b0100_0000,
    // bit 5 not used
    Absolute = 0b0001_0000, // ABS (Absolute [min..max])
    Relative = 0b0000_1000, // REL (Relative [-1..1])
    Discrete = 0b0000_0100, // DIS (Discrete [0, 1, 2, 3])
                            // bit 1 not used
                            // bit 0 not used
}
