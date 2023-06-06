// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

#![allow(unused)]
use core::time::Duration;
use tokio::time::sleep as sleep;

use lego_powered_up::setup;
use lego_powered_up::{PoweredUp, ConnectedHub, IoDevice, IoTypeId}; 
use lego_powered_up::{Hub, HubFilter, }; 
use lego_powered_up::error::{Error, Result, OptionContext}; 
use lego_powered_up::consts::named_port;
use lego_powered_up::notifications::*;
use lego_powered_up::consts::{LEGO_COLORS, };
use lego_powered_up::iodevice::modes;
use lego_powered_up::iodevice::remote::{RcDevice, RcButtonState};
use lego_powered_up::iodevice::{hubled::*, sensor::*, motor::*};

use lego_powered_up::notifications::{NotificationMessage, ModeInformationRequest, ModeInformationType,
    InformationRequest, InformationType, HubAction, HubActionRequest, InputSetupSingle,
    PortValueSingleFormat, PortValueCombinedFormat, NetworkCommand};
use lego_powered_up::consts::HubPropertyOperation;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = setup::single_hub().await?;

    // Hub actions
    // {
    //     let lock = hub.mutex.lock().await;
    //     println!("Busy!");
    //     lock.hub_action(lego_powered_up::notifications::HubAction::ActivateBusyIndication).await;
    //     sleep(Duration::from_secs(3)).await;
    //     println!("Not busy");
    //     lock.hub_action(lego_powered_up::notifications::HubAction::ResetBusyIndication).await;
    //     sleep(Duration::from_secs(3)).await;
    //     println!("Switch off");
    //     lock.hub_action(lego_powered_up::notifications::HubAction::SwitchOffHub).await;
    //     sleep(Duration::from_secs(3)).await;
    // }

    // Hub properties
    {
        // let lock = hub.mutex.lock().await;
        // lock.hub_props(HubPropertyValue::AdvertisingName(Vec::new()), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::Button(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::FwVersion(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::HwVersion(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::Rssi(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::BatteryVoltage(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::BatteryType(HubBatteryType::Normal), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::ManufacturerName(Vec::new()), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::RadioFirmwareVersion(Vec::new()), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::LegoWirelessProtocolVersion(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::SystemTypeId(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::HwNetworkId(0), HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::PrimaryMacAddress([0;6]), HubPropertyOperation::RequestUpdateDownstream).await;
        // // lock.hub_prop_req(HubPropertyValue::SecondaryMacAddress, HubPropertyOperation::RequestUpdateDownstream).await;
        // lock.hub_props(HubPropertyValue::HardwareNetworkFamily(0), HubPropertyOperation::RequestUpdateDownstream).await;
    }

    // Hub alerts
    {
        let lock = hub.mutex.lock().await;
        lock.hub_alerts(AlertType::LowVoltage, AlertOperation::RequestUpdate).await;
        lock.hub_alerts(AlertType::HighCurrent, AlertOperation::RequestUpdate).await;
        lock.hub_alerts(AlertType::LowSignalStrength, AlertOperation::RequestUpdate).await;
        lock.hub_alerts(AlertType::OverPowerCondition, AlertOperation::RequestUpdate).await;
    }


    sleep(Duration::from_secs(10)).await;


    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    Ok(())
}

