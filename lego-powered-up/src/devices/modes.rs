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
    
}
pub mod TechnicLargeLinearMotorMoveHub {

}
pub mod TechnicLargeLinearMotorTechnicHub {

}
pub mod TechnicXLargeLinearMotorMoveHub {

}
pub mod TechnicXLargeLinearMotorTechnicHub {

}
pub mod TechnicHubGestSensor {

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

}
pub mod TechnicHubGyroSensor {

}
pub mod TechnicHubTiltSensor {

}
pub mod TechnicHubTemperatureSensor {

}
pub mod UnknownMovehubDevice {
    pub const TRIGGER: u8 = 0; 
    pub const CANVAS:u8 = 1;
    pub const VAR: u8 = 2;
}

//    pub const : u8 = ;