#[derive(Debug, Copy, Clone)]
pub struct RemoteStatus {
    // Buttons
    pub a_plus: bool,
    pub a_red: bool,
    pub a_minus: bool,
    pub green: bool,
    pub b_plus: bool,
    pub b_red: bool,
    pub b_minus: bool,
    // Operaional status
    pub battery: u8,   // 0 - 100 %
    pub rssi: i8,      // -127 - 0  
}

impl RemoteStatus {
    pub fn new() -> Self {
        Self {
            a_plus: false,
            a_red: false,
            a_minus: false,
            green: false,
            b_plus: false,
            b_red: false,
            b_minus: false,
            battery: 100,
            rssi: 0
        }

    }
}
