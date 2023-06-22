// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

// #![allow(unused)]
use core::time::Duration;
use tokio::time::sleep;

use lego_powered_up::consts::{HubPropertyOperation, HubPropertyRef};
use lego_powered_up::notifications::*;
use lego_powered_up::setup;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hub = setup::single_hub().await?;

    // Hub properties
    {
        let lock = hub.mutex.lock().await;
        lock.hub_props(
            HubPropertyRef::AdvertisingName,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::Button,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::FwVersion,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::HwVersion,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::Rssi,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::BatteryVoltage,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::BatteryType,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::ManufacturerName,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::RadioFirmwareVersion,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::LegoWirelessProtocolVersion,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::SystemTypeId,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::HwNetworkId,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        lock.hub_props(
            HubPropertyRef::PrimaryMacAddress,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
        // lock.hub_prop_req(HubPropertyRef::SecondaryMacAddress, HubPropertyOperation::RequestUpdateDownstream).await?;
        lock.hub_props(
            HubPropertyRef::HardwareNetworkFamily,
            HubPropertyOperation::RequestUpdateDownstream,
        )?;
        // .await?;
    }
    sleep(Duration::from_secs(1)).await;

    // Hub alerts
    {
        let lock = hub.mutex.lock().await;
        lock.hub_alerts(AlertType::LowVoltage, AlertOperation::RequestUpdate)?;
            // .await?;
        lock.hub_alerts(AlertType::HighCurrent, AlertOperation::RequestUpdate)?;
            // .await?;
        lock.hub_alerts(
            AlertType::LowSignalStrength,
            AlertOperation::RequestUpdate,
        )?;
        // .await?;
        lock.hub_alerts(
            AlertType::OverPowerCondition,
            AlertOperation::RequestUpdate,
        )?;
        // .await?;
    }
    sleep(Duration::from_secs(1)).await;

    // Hub actions
    {
        let lock = hub.mutex.lock().await;
        println!("Busy!");
        lock.hub_action(
            lego_powered_up::notifications::HubAction::ActivateBusyIndication,
        )?;
        // .await?;
        sleep(Duration::from_secs(3)).await;
        println!("Not busy");
        lock.hub_action(
            lego_powered_up::notifications::HubAction::ResetBusyIndication,
        )?;
        // .await?;
        sleep(Duration::from_secs(3)).await;
        println!("Switch off");
        lock.hub_action(
            lego_powered_up::notifications::HubAction::SwitchOffHub,
        )?;
        // .await?;
        sleep(Duration::from_secs(3)).await;

        // lock.hub_action(lego_powered_up::notifications::HubAction::Disconnect).await?;
        // lock.hub_action(lego_powered_up::notifications::HubAction::VccPortControlOn).await?;
        // lock.hub_action(lego_powered_up::notifications::HubAction::VccPortControlOff).await?;
    }

    // Cleanup
    println!("Disconnect from hub `{}`", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }

    Ok(())
}
