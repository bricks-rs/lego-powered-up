// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

use std::collections::HashMap;
use text_io::read;
use tokio::task::JoinHandle;

use lego_powered_up::error::Result;
use lego_powered_up::iodevice::basic::Basic;
use lego_powered_up::iodevice::modes;
use lego_powered_up::iodevice::{hubled::*, sensor::*, visionsensor::*};
use lego_powered_up::notifications::DatasetType;
use lego_powered_up::setup;
use lego_powered_up::HubMutex;
use lego_powered_up::{IoDevice, IoTypeId};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = setup::single_hub().await?;

    vision_sensor_ui(hub.mutex.clone()).await?;

    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    Ok(())
}

pub async fn vision_sensor_ui(mutex: HubMutex) -> Result<()> {
    let mut tasks: std::collections::HashMap<u8, JoinHandle<()>> =
        HashMap::new();
    let device: IoDevice;
    {
        let lock = mutex.lock().await;
        device = lock
            .io_from_kind(IoTypeId::VisionSensor)
            .expect("Can't access VisionSensor");
    }
    let port_id = device.def.port();
    let delta = 1u32;
    loop {
        print!("(l)ist, <mode>, or (q)uit > ");
        let line: String = read!("{}\n");
        if line.is_empty() || line.starts_with('\r') {
            if let Some(task) = tasks.remove(&port_id) {
                task.abort();
            }
            device
                .device_mode(0, 1, false)
                .await
                .expect("Error disabling notifications");
            continue;
        } else if line.trim().eq_ignore_ascii_case("color") {
            if let Some(task) = tasks.insert(
                port_id,
                vision_to_hub_color(&device, mutex.clone()).await?,
            ) {
                task.abort();
            }
        } else if line.trim().eq_ignore_ascii_case("prox") {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::PROX,
                    delta,
                    String::from("PROX"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if line.trim().eq_ignore_ascii_case("count") {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::COUNT,
                    delta,
                    String::from("COUNT"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if line.trim().eq_ignore_ascii_case("reflt") {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::REFLT,
                    delta,
                    String::from("REFLT"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if line.trim().eq_ignore_ascii_case("ambi") {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::AMBI,
                    delta,
                    String::from("AMBI"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if line.trim().eq_ignore_ascii_case("col_o")
            || line.trim().eq_ignore_ascii_case("col o")
        {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::COL_O,
                    delta,
                    String::from("COL_O"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if (line.trim().eq_ignore_ascii_case("rgb_i"))
            || (line.trim().eq_ignore_ascii_case("rgb i"))
        {
            if let Some(task) = tasks.insert(
                port_id,
                vision_to_hub_rgb(&device, mutex.clone()).await?,
            ) {
                task.abort();
            }
        } else if (line.trim().eq_ignore_ascii_case("ir_tx"))
            || (line.trim().eq_ignore_ascii_case("ir tx"))
        {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::IR_TX,
                    delta,
                    String::from("IR_Tx"),
                )
                .await?,
            ) {
                task.abort()
            }
        } else if (line.trim().eq_ignore_ascii_case("spec_1"))
            || (line.trim().eq_ignore_ascii_case("spec 1"))
        {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::SPEC_1,
                    delta,
                    String::from("SPEC_1"),
                )
                .await?,
            ) {
                task.abort()
            }
        } else if line.trim().eq_ignore_ascii_case("debug") {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::DEBUG,
                    delta,
                    String::from("DEBUG"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if line.trim().eq_ignore_ascii_case("calib") {
            if let Some(task) = tasks.insert(
                port_id,
                reader(
                    &device,
                    port_id,
                    modes::VisionSensor::CALIB,
                    delta,
                    String::from("CALIB"),
                )
                .await?,
            ) {
                task.abort();
            }
        } else if line.contains('q') {
            break;
        } else if line.contains('l') {
            for m in device.def.modes().values() {
                println!("{}", m);
            }
            continue;
        }
    }
    Ok(())
}

async fn reader(
    device: &IoDevice,
    port_id: u8,
    mode_id: u8,
    delta: u32,
    name: String,
) -> Result<JoinHandle<()>> {
    match device
        .def
        .modes()
        .get(&mode_id)
        .unwrap()
        .value_format
        .dataset_type
    {
        // panics on non-existant mode_id
        DatasetType::Bits8 => {
            let (mut rx, _) =
                device.enable_8bit_sensor(mode_id, delta).await.unwrap();
            Ok(tokio::spawn(async move {
                while let Ok(data) = rx.recv().await {
                    println!(
                        "Port {:?} mode {:} sent: {:?}",
                        port_id, name, &data
                    );
                }
            }))
        }
        DatasetType::Bits16 => {
            let (mut rx, _) =
                device.enable_16bit_sensor(mode_id, delta).await.unwrap();
            Ok(tokio::spawn(async move {
                while let Ok(data) = rx.recv().await {
                    println!(
                        "Port {:?} mode {:} sent: {:?}",
                        port_id, name, &data
                    );
                }
            }))
        }
        DatasetType::Bits32 => {
            let (mut rx, _) =
                device.enable_32bit_sensor(mode_id, delta).await.unwrap();
            Ok(tokio::spawn(async move {
                while let Ok(data) = rx.recv().await {
                    println!(
                        "Port {:?} mode {:} sent: {:?}",
                        port_id, name, &data
                    );
                }
            }))
        }
        DatasetType::Float => {
            let (mut rx, _) =
                device.enable_32bit_sensor(mode_id, delta).await.unwrap();
            Ok(tokio::spawn(async move {
                while let Ok(data) = rx.recv().await {
                    println!(
                        "Port {:?} mode {:} sent: {:?}",
                        port_id, name, &data
                    );
                }
            }))
        }
    }
}

async fn vision_to_hub_color(
    device: &IoDevice,
    mutex: HubMutex,
) -> Result<JoinHandle<()>> {
    let hubled: IoDevice;
    {
        let lock = mutex.lock().await;
        hubled = lock
            .io_from_kind(IoTypeId::HubLed)
            // .await
            .expect("Can't access Hubled");
    }
    let (mut vision_rx, _) = device.visionsensor_color().await.unwrap();
    hubled
        .set_hubled_mode(HubLedMode::Colour)
        .await
        .expect("Error setting mode");
    Ok(tokio::spawn(async move {
        while let Ok(data) = vision_rx.recv().await {
            println!("Color: {:?} ", data,);
            match data {
                DetectedColor::NoObject => {
                    hubled.set_hubled_color(Color::Black).await.unwrap();
                }
                DetectedColor::Black => {
                    hubled.set_hubled_color(Color::Black).await.unwrap();
                }
                DetectedColor::Blue => {
                    hubled.set_hubled_color(Color::Blue).await.unwrap();
                }
                DetectedColor::Green => {
                    hubled.set_hubled_color(Color::Green).await.unwrap();
                }
                DetectedColor::Yellow => {
                    hubled.set_hubled_color(Color::Yellow).await.unwrap();
                }
                DetectedColor::Red => {
                    hubled.set_hubled_color(Color::Red).await.unwrap();
                }
                DetectedColor::White => {
                    hubled.set_hubled_color(Color::White).await.unwrap();
                }
                _ => (),
            }
        }
    }))
}

async fn vision_to_hub_rgb(
    device: &IoDevice,
    mutex: HubMutex,
) -> Result<JoinHandle<()>> {
    let hubled: IoDevice;
    {
        let lock = mutex.lock().await;
        hubled = lock
            .io_from_kind(IoTypeId::HubLed)
            .expect("Can't access Hubled");
    }
    hubled
        .set_hubled_mode(HubLedMode::Rgb)
        .await
        .expect("Error setting mode");
    hubled.set_hubled_rgb(&[0x00, 0x00, 0x00]).await.unwrap();
    let (mut vision_rx, _) = device
        .enable_16bit_sensor(modes::VisionSensor::RGB_I, 1)
        .await
        .unwrap();
    Ok(tokio::spawn(async move {
        while let Ok(data) = vision_rx.recv().await {
            hubled
                .set_hubled_rgb(&[data[0] as u8, data[1] as u8, data[2] as u8])
                .await
                .unwrap();
            println!("RGB: {:?} ", data,)
        }
    }))
}
