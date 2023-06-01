#![allow(non_snake_case)]
pub mod Voltage {
    pub const VLT_L: u8 = 0;
    pub const VLT_S: u8 = 1;
}
pub mod Current {
    pub const CUR_L: u8 = 0;
    pub const CUR_S: u8 = 1;
}
pub mod HubLed  {
    pub const COL_O: u8 = 0;
    pub const RGB_O: u8 = 1;
}
pub mod VisionSensor {
    pub const COLOR: u8 = 0;
    pub const PROX: u8 = 1;
    pub const COUNT: u8 = 2;
    pub const REFLT: u8 = 3;
    pub const AMBI: u8 = 4;
    pub const COL_O: u8 = 5;
    pub const RGB_I: u8 = 6;
    pub const IR_TX: u8 = 7;
    pub const SPEC_1: u8 = 8;
    pub const DEBUG: u8 = 9;
    pub const CALIB: u8 = 10;
}
pub mod InternalMotorTacho {
    pub const POWER: u8 = 0;
    pub const SPEED: u8 = 1;
    pub const POS: u8 = 2;
}
pub mod InternalTilt {
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    // "ANGLE",
    // "TILT",
    // "ORINT",
    // "IMPCT",
    // "ACCEL",
    // "OR_CF",
    // "IM_CF",
    // "CALIB",
}
pub mod TechnicLargeLinearMotorMoveHub {
//     "POWER",
//     "SPEED",
//     "POS",
//     "APOS",
//     "CALIB",
//     "STATS",
}
pub mod TechnicLargeLinearMotorTechnicHub {
    // "POWER",
    // "SPEED",
    // "POS",
    // "APOS",
    // "LOAD",
}
pub mod TechnicXLargeLinearMotorMoveHub {
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
//     "POWER",
//     "SPEED",
//     "POS",
//     "APOS",
//     "CALIB",
//     "STATS",
}
pub mod TechnicXLargeLinearMotorTechnicHub {
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    // "POWER",
    // "SPEED",
    // "POS",
    // "APOS",
    // "LOAD",
}
pub mod TechnicHubGestSensor {
    //    pub const : u8 = ;
    // "GEST",
}
pub mod RemoteButtons {
    pub const RCKEY: u8 = 0;
    pub const KEYA: u8 = 1;
    pub const KEYR: u8 = 2;
    pub const KEYD: u8 = 3;
    pub const KEYSD: u8 = 4;
}
pub mod RemoteRssi {
    pub const RSSI: u8 = 0;
}
pub mod TechnicHubAccelerometer {
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    // "GRV",
    // "CAL",
}
pub mod TechnicHubGyroSensor {
    pub const ROT: u8 = 0;
}
pub mod TechnicHubTiltSensor {
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    //    pub const : u8 = ;
    // "POS",
    // "IMP",
    // "CFG",
}
pub mod TechnicHubTemperatureSensor {
    pub const TEMP: u8 = 0; 
}
pub mod UnknownMovehubDevice {
    pub const TRIGGER: u8 = 0; 
    pub const CANVAS:u8 = 1;
    pub const VAR: u8 = 2;
}

//    pub const : u8 = ;