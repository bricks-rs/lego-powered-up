#![allow(dead_code)]
use btleplug::api::PeripheralProperties;
#[allow(unused_imports)]
use rand::{thread_rng, Rng};

#[allow(unused_imports)]
use std::thread;


#[allow(unused_imports)]
use btleplug::api::{Central, CentralEvent, Characteristic, Peripheral};
#[allow(unused_imports)]
#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::Adapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};

use num_traits::FromPrimitive;

use consts::*;
mod consts;

#[cfg(target_os = "linux")]
fn print_adapter_info(adapter: &Adapter) {
    println!(
        "connected adapter {:?} is powered: {:?}",
        adapter.name(),
        adapter.is_powered()
    );
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn print_adapter_info(_adapter: &Adapter) {
    println!("adapter info can't be printed on Windows 10 or mac");
}

fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().unwrap();
    adapters.into_iter().next().unwrap()
}

/**
If you are getting run time error like that :
 thread 'main' panicked at 'Can't scan BLE adapter for connected devices...: PermissionDenied', src/libcore/result.rs:1188:5
 you can try to run app with > sudo ./discover_adapters_peripherals
 on linux
**/
fn main() {
    let manager = Manager::new().unwrap();
    let adapter = get_central(&manager);
    println!("connecting to BLE adapter: ...");

    print_adapter_info(&adapter);
    let event_rx = adapter
        .event_receiver()
        .expect("Can't scan BLE adapter for connected devices...");

    // start the scan
    adapter.start_scan().unwrap();

    while let Ok(evt) = event_rx.recv() {
        // all peripheral devices in range
        //for peripheral in adapter.peripherals().iter() {
        println!("Event: {:?}", evt);
        if let CentralEvent::DeviceDiscovered(dev) = evt {
            let peripheral = adapter.peripheral(dev).unwrap();
            println!(
                "peripheral : {:?} is connected: {:?}",
                peripheral.properties().local_name,
                peripheral.is_connected()
            );
            if peripheral.properties().local_name.is_some()
                && !peripheral.is_connected()
            {
                if let Some(hub_type) = peripheral.identify() {
                    println!("Looks like a '{:?}' hub!", hub_type);
                } else {
                	println!("Device does not look like a PoweredUp Hub");
                }

                //let hub = register_hub(&peripheral.properties());
                /*println!(
                    "start connect to peripheral : {:?}...",
                    peripheral.properties().local_name
                );
                peripheral
                    .connect()
                    .expect("Can't connect to peripheral...");
                println!(
                    "now connected (\'{:?}\') to peripheral : {:?}...",
                    peripheral.is_connected(),
                    peripheral.properties().local_name
                );
                let chars = peripheral.discover_characteristics();
                if peripheral.is_connected() {
                    println!(
                        "Discover peripheral : \'{:?}\' characteristics...",
                        peripheral.properties().local_name
                    );
                    for chars_vector in chars.into_iter() {
                        for char_item in chars_vector.iter() {
                            println!("{:?}", char_item);
                        }
                    }
                    println!(
                        "disconnecting from peripheral : {:?}...",
                        peripheral.properties().local_name
                    );
                    peripheral.disconnect().expect(
                        "Error on disconnecting from BLE peripheral ",
                    );
                }*/
                eprintln!(
                    "Not connecting because don't want to interrupt denon"
                );
            } else {
                //sometimes peripheral is not discovered completely
                eprintln!(
                    "SKIP connect to UNKNOWN peripheral : {:?}",
                    peripheral
                );
            }
        }
    }
}

pub fn register_hub(peripheral: &PeripheralProperties) -> Option<impl Hub> {
    Option::<TechnicHub>::None
}

pub trait Hub {
    fn name(&self) -> &str;
}

pub struct TechnicHub;

impl Hub for TechnicHub {
    fn name(&self) -> &str {
        "game"
    }
}

pub trait IdentifyHub {
    fn identify(&self) -> Option<HubType>;
}

/*
PeripheralProperties 
{ 
 address: 90:84:2B:60:3C:B8,
 address_type: Public,
 local_name: Some("game"),
 tx_power_level: Some(-66),
 manufacturer_data: {919: [0, 128, 6, 0, 97, 0]},
 service_data: {}, 
 services: [00001623-1212-efde-1623-785feabcd123], 
 discovery_count: 1, 
 has_scan_response: false
}
*/
impl<P: Peripheral> IdentifyHub for P {
    fn identify(&self) -> Option<HubType> {
        use HubType::*;

        let props = self.properties();
        println!("props:\n{:?}", props);

        if props
            .services
            .contains(&consts::bleservice::WEDO2_SMART_HUB)
        {
            return Some(Wedo2SmartHub);
        } else if props.services.contains(&consts::bleservice::LPF2_HUB) {
            if let Some(manufacturer_id) = props.manufacturer_data.get(&919) {
                // Can't do it with a match because some devices are just manufacturer
                // data while some use other characteristics
                if let Some(m) =
                    BLEManufacturerData::from_u8(manufacturer_id[1])
                {
                    use BLEManufacturerData::*;
                    return Some(match m {
                        DuploTrainBaseId => DuploTrainBase,
                        HubId => Hub,
                        MarioId => Mario,
                        MoveHubId => MoveHub,
                        RemoteControlId => RemoteControl,
                        TechnicMediumHubId => TechnicMediumHub,
                    });
                }
            }
        }
        None
    }
}
