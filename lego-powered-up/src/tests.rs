#![allow(unused)]
#[cfg(test)]

mod tests {
    use crate::consts::IoTypeId;
    use crate::hubs;
    use crate::hubs::generic_hub::GenericHub;
    use crate::iodevice::IoDevice;
    use crate::HubType;
    use btleplug::api::{Characteristic, Peripheral as _, WriteType};
    use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
    use std::collections::BTreeMap;

    #[test]
    fn test_get_from_port() {
        // setup
        let mut io_devices: BTreeMap<u8, IoDevice> = BTreeMap::new();
        // let io_device = IoDevice::new(IoTypeId::HubLed, 0x34);
        // io_devices.insert(0x34, io_device);


        // let a = Adapter::new

        // let p = Peripheral {

        // }
        // hub = GenericHub {

        // }
    }
}
