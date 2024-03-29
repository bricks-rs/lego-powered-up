//! # Specific implementations for each of the supported hubs.
//1 Models a hub with hub-related properties and commands, as well as
//! accessing connected devices (internal and external).
//!
//! Accessing devices through the hub has changed; instead of a fixed port map,
//! the map connected_io is populated with attached devices and their available
//! options and we can select a device from there.
//! The io_from_.. methods wrap some useful calls on connected_io(), for example;
//! io_from_kind(IoTypeId::HubLed)
//! accesses the LED on any hub type though hardware addresses differ,
//! io_multiple_from_kind(IoTypeId::Motor)
//! accesses all motors indifferent to where they are connected.
//!
//! This also reduces the need for specific implementations, all three
//! types I have available are supported by generic_hub.  

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::consts::{HubPropertyOperation, HubPropertyRef, HubType};
use crate::error::{Error, OptionContext, Result};
use crate::notifications::{
    AlertOperation, AlertPayload, AlertType, ErrorMessageFormat, HubAction,
    HubActionRequest, HubAlert, HubProperty, HubPropertyValue,
    InformationRequest, InformationType, InputSetupSingle,
    ModeInformationRequest, ModeInformationType, NetworkCommand,
    NotificationMessage, PortOutputCommandFeedbackFormat,
    PortValueCombinedFormat, PortValueSingleFormat,
};
use crate::{IoDevice, IoTypeId};
pub type Tokens = Arc<(Peripheral, Characteristic)>;

pub mod generic_hub;
pub mod io_event;

/// Trait describing a generic hub.
#[async_trait::async_trait]
pub trait Hub: Debug + Send + Sync {
    async fn name(&self) -> Result<String>;
    async fn disconnect(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn is_connected(&self) -> Result<bool>;
    // The init function cannot be a trait method until we have GAT :(
    //fn init(peripheral: P);
    fn properties(&self) -> &HubProperties;
    fn kind(&self) -> HubType;
    fn connected_io(&self) -> &BTreeMap<u8, IoDevice>;
    fn connected_io_mut(&mut self) -> &mut BTreeMap<u8, IoDevice>;
    fn channels(&mut self) -> &mut crate::hubs::Channels;
    // fn detach_io(&mut self, ) -> Result<()>;
    async fn subscribe(&self, char: Characteristic) -> Result<()>;
    fn io_from_port(&self, port_id: u8) -> Result<IoDevice>;
    fn io_from_kind(&self, kind: IoTypeId) -> Result<IoDevice>;
    fn io_multi_from_kind(&self, kind: IoTypeId) -> Result<Vec<IoDevice>>;

    fn tokens(&self) -> Tokens;
    fn attach_io(&mut self, io_type_id: IoTypeId, port_id: u8) -> Result<()>;
    fn peripheral(&self) -> Arc<Peripheral>;
    fn characteristic(&self) -> Arc<Characteristic>;
    fn device_cache(&self, d: IoDevice) -> IoDevice;
    fn cancel_token(&self) -> CancellationToken;

    // Port information
    async fn request_port_info(
        &self,
        port_id: u8,
        infotype: InformationType,
    ) -> Result<()> {
        let msg =
            NotificationMessage::PortInformationRequest(InformationRequest {
                port_id,
                information_type: infotype,
            });
        self.send(msg).await
    }
    async fn req_mode_info(
        &self,
        port_id: u8,
        mode: u8,
        infotype: ModeInformationType,
    ) -> Result<()> {
        let msg = NotificationMessage::PortModeInformationRequest(
            ModeInformationRequest {
                port_id,
                mode,
                information_type: infotype,
            },
        );
        self.send(msg).await
    }

    async fn set_port_mode(
        &self,
        port_id: u8,
        mode: u8,
        delta: u32,
        notification_enabled: bool,
    ) -> Result<()> {
        let msg =
            NotificationMessage::PortInputFormatSetupSingle(InputSetupSingle {
                port_id,
                mode,
                delta,
                notification_enabled,
            });
        self.send(msg).await
    }

    /// Hub properties: Single request, enable/disable notifications, reset
    async fn hub_props(
        &self,
        reference: HubPropertyRef,
        operation: HubPropertyOperation,
    ) -> Result<()> {
        let msg = NotificationMessage::HubProperties(HubProperty {
            reference,
            operation,
            property: HubPropertyValue::SecondaryMacAddress, // Not used in request
        });
        self.send(msg).await
    }

    /// Perform Hub actions
    async fn hub_action(&self, action_type: HubAction) -> Result<()> {
        let msg =
            NotificationMessage::HubActions(HubActionRequest { action_type });
        self.send(msg).await
    }

    /// Hub alerts: Single request, enable/disable notifications
    async fn hub_alerts(
        &self,
        alert_type: AlertType,
        operation: AlertOperation,
    ) -> Result<()> {
        let msg = NotificationMessage::HubAlerts(HubAlert {
            alert_type,
            operation,
            payload: AlertPayload::StatusOk,
        });
        self.send(msg).await
    }

    async fn send(&self, msg: NotificationMessage) -> Result<()> {
        let buf = msg.serialise();
        let tokens = self.tokens();
        tokens
            .0
            .write(&tokens.1, &buf, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }

    // Cannot provide a default implementation without access to the Peripheral trait from here
    async fn send_raw(&self, msg: &[u8]) -> Result<()>;
}

pub type VersionNumber = u8;
/// Propeties of a hub
#[derive(Debug, Default)]
pub struct HubProperties {
    /// Friendly name, set via the PoweredUp or Control+ apps
    pub name: String,
    /// Firmware revision
    pub fw_version: String,
    /// Hardware revision
    pub hw_version: String,
    /// BLE MAC address
    pub mac_address: String,
    /// Battery level
    pub battery_level: usize,
    /// BLE signal strength
    pub rssi: i16,
}

/// Devices can use this with cached tokens and not need to mutex-lock hub
pub async fn send(tokens: Tokens, msg: NotificationMessage) -> Result<()> {
    let buf = msg.serialise();
    tokens
        .0
        .write(&tokens.1, &buf, WriteType::WithoutResponse)
        .await?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct Channels {
    pub singlevalue_sender:
        Option<tokio::sync::broadcast::Sender<PortValueSingleFormat>>,
    pub combinedvalue_sender:
        Option<tokio::sync::broadcast::Sender<PortValueCombinedFormat>>,
    pub networkcmd_sender:
        Option<tokio::sync::broadcast::Sender<NetworkCommand>>,
    pub hubnotification_sender:
        Option<tokio::sync::broadcast::Sender<HubNotification>>,
    pub commandfeedback_sender:
        Option<tokio::sync::broadcast::Sender<PortOutputCommandFeedbackFormat>>,
}

#[derive(Debug, Default, Clone)]
pub struct HubNotification {
    pub hub_property: Option<HubProperty>,
    pub hub_action: Option<HubActionRequest>,
    pub hub_alert: Option<HubAlert>,
    pub hub_error: Option<ErrorMessageFormat>,
}
