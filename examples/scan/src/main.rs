// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral as _, PeripheralProperties,
    ScanFilter, 
    ValueNotification
};
use btleplug::platform::{Adapter, Manager, PeripheralId, };

use lego_powered_up::error::{Error, Result, OptionContext};

use core::pin::Pin;
use futures::stream::{Stream, StreamExt};
use lego_powered_up::DiscoveredHub;


use lego_powered_up::{PoweredUp, HubFilter, ConnectedHub, IoDevice}; 
use lego_powered_up::consts::named_port;
use lego_powered_up::iodevice::remote::RcDevice;
use lego_powered_up::iodevice::remote::RcButtonState;

type DiscoStream = Pin<Box<dyn Stream<Item = DiscoveredHub> + Send>>;
// type DiscoStream2 = dyn Stream<Item = DiscoveredHub> + Send;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut pu: PoweredUp = PoweredUp::init().await?;


    let scan_stream = pu.scan().await?;
    // let mut scan_stream: &'static Pin<Box<dyn Stream<Item = DiscoveredHub> + Send>> = &pu.scan2().await?;
    // let mut scan_stream: &'static Pin<Box<dyn Stream<Item = DiscoveredHub> + Send>> = &pu.scan2().await?;
    // let mut scan_stream: Pin<Box<dyn Stream<Item = DiscoveredHub> + Send +'static>> = pu.scan2().await?;
    // let mut scan_stream: Pin<Box<dyn Stream<Item = DiscoveredHub> + Send >> = pu.scan2().await?;

    let scan_task = tokio::spawn(async move {
        // scanner(Box::pin(scan_stream)).await.expect("Stream error");
        // scanner(scan_stream).await.expect("Stream error");
        
    });


    
    println!("Done!");
    Ok(())
}

// pub async fn scanner<'a>(stream: &'a PeriStream ) -> Result<()> {
pub async fn scanner(mut stream: DiscoStream ) -> Result<()> {
    while let Some(data) = stream.next().await {
        dbg!(data);
    }
    println!("DiscoHub stream ended");
    Ok(())
}
