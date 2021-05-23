use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use lego_powered_up::{BDAddr, DiscoveredHub, HubController, PoweredUp};
use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
};

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<UiState>()
        .add_plugins(DefaultPlugins)
        //.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_startup_system(startup_system.system())
        .add_system(update_ui_scale_factor.system())
        .add_system(ui.system())
        .add_system(update_discovered_hubs.system())
        .run();
}

#[derive(Default)]
struct UiState {
    label: String,
    devices: Vec<DeviceInfo>,
    selected_device_idx: usize,
    available_hubs: Vec<DiscoveredHub>,
    connected_hubs: HashMap<BDAddr, HubController>,
    connection_state: ConnectionState,
    gui_selection: GuiSelection,
    connect_custom_address: String,
}

impl UiState {
    pub fn pu(&self) -> Option<&PoweredUp> {
        if let ConnectionState::Connected(pu) = &self.connection_state {
            Some(pu)
        } else {
            None
        }
    }
}

struct HubInfo {
    addr: BDAddr,
}

struct DeviceInfo {
    idx: usize,
    name: String,
}

enum ConnectionState {
    NotConnected,
    Connected(PoweredUp),
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::NotConnected
    }
}

impl ConnectionState {
    pub fn take(&mut self) -> Option<PoweredUp> {
        use ConnectionState::*;
        if let Connected(pu) = std::mem::take(self) {
            Some(pu)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum GuiSelection {
    None,
    DiscoveredHub(BDAddr),
    ConnectedHub(BDAddr),
}

impl Default for GuiSelection {
    fn default() -> Self {
        GuiSelection::None
    }
}

impl GuiSelection {
    pub fn is_none(&self) -> bool {
        *self == GuiSelection::None
    }

    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

fn update_discovered_hubs(mut ui_state: ResMut<UiState>) {
    if let ConnectionState::Connected(pu) = &ui_state.connection_state {
        let hubs = pu.list_discovered_hubs().unwrap();
        ui_state.available_hubs = hubs;
    }
}

fn startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ui_state: ResMut<UiState>,
) {
    // Populate the devices list
    PoweredUp::devices()
        .unwrap()
        .iter()
        .enumerate()
        .for_each(|(idx, dev)| {
            ui_state.devices.push(DeviceInfo {
                idx,
                name: dev.name().unwrap_or_else(|_| "unknown".to_string()),
            });
        });
    info!("Found {} Bluetooth devices", ui_state.devices.len());
}

fn update_ui_scale_factor(
    mut egui_settings: ResMut<EguiSettings>,
    windows: Res<Windows>,
) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}

fn ui(egui_ctx: ResMut<EguiContext>, mut ui_state: ResMut<UiState>) {
    use ConnectionState::*;

    // Menu bar
    egui::TopPanel::top("Top panel").show(egui_ctx.ctx(), |ui| {
        egui::menu::bar(ui, |ui| {
            // FILE menu
            egui::menu::menu(ui, "File", |ui| {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            // Drop-down selection for bluetooth interfaces
            if !ui_state.devices.is_empty() {
                let mut selected_device_idx = ui_state.selected_device_idx;
                egui::ComboBox::from_label("").show_index(
                    ui,
                    &mut selected_device_idx,
                    ui_state.devices.len(),
                    |idx| ui_state.devices[idx].name.clone(),
                );
                ui_state.selected_device_idx = selected_device_idx;
            } else {
                ui.label("No Bluetooth devices");
            }

            // Button to start & stop BLE
            let mut do_disconnect = false;
            match &ui_state.connection_state {
                NotConnected => {
                    if ui.button("Start BLE").clicked() {
                        info!(
                            "Connecting to device {}",
                            ui_state.selected_device_idx
                        );
                        match PoweredUp::with_device(
                            ui_state.selected_device_idx,
                        ) {
                            Ok(pu) => {
                                // Successfully created the BLE background process
                                ui_state.connection_state =
                                    ConnectionState::Connected(pu);
                            }
                            Err(e) => {
                                error!("Error starting BLE process: {}", e);
                            }
                        }
                    }
                }
                Connected(_) => {
                    if ui.button("Stop BLE").clicked() {
                        do_disconnect = true;
                    }
                }
            }
            if do_disconnect {
                if let Some(mut pu) = ui_state.connection_state.take() {
                    info!("Shutting down BLE process");
                    if let Err(e) = pu.stop() {
                        error!("Error shutting down BLE process: {}", e);
                    }
                    ui_state.connection_state = NotConnected;
                }
            }
        });
    });

    // Left panel
    egui::SidePanel::left("side_panel", 300.0).show(egui_ctx.ctx(), |ui| {
        // List of hubs
        ui.heading("Discovered hubs:");
        let mut update_gui_selection = GuiSelection::None;
        for hub in &ui_state.available_hubs {
            ui.horizontal(|ui| {
                if ui.button(format!("{}: {}", hub.name, hub.addr)).clicked() {
                    // Set "selected gui element" to this DiscoveredHub instance
                    update_gui_selection =
                        GuiSelection::DiscoveredHub(hub.addr);
                }
            });
        }

        // Textbox for connecting to a hub by address
        ui.text_edit_singleline(&mut ui_state.connect_custom_address);
        if ui.button("Connect").clicked() {
            // Connect to this address
            if let Connected(pu) = &ui_state.connection_state {
                match pu.connect_to_hub(&ui_state.connect_custom_address) {
                    Ok(hub) => {
                        info!("Connected to hub {}", hub.get_addr());
                        update_gui_selection =
                            GuiSelection::ConnectedHub(*hub.get_addr());
                        ui_state.connected_hubs.insert(*hub.get_addr(), hub);
                    }
                    Err(e) => {
                        error!(
                            "Error connecting to hub {}: {}",
                            ui_state.connect_custom_address, e
                        );
                    }
                }
            }
        }
        if !update_gui_selection.is_none() {
            ui_state.gui_selection = update_gui_selection.take();
        }

        // Details pane
        ui.heading("Details");
        match ui_state.gui_selection {
            GuiSelection::None => {
                ui.label("Nothing selected");
            }
            GuiSelection::DiscoveredHub(addr) => {
                let details =
                    ui_state.available_hubs.iter().find(|h| h.addr == addr);
                match details {
                    None => {
                        ui.label(format!("Invalid selection {}", addr));
                    }
                    Some(hub) => {
                        ui.label(hub.name.clone());
                        ui.label(hub.hub_type.to_string());
                        ui.label(hub.addr.to_string());

                        // Connect button
                        if ui.button("Connect").clicked() {
                            if let Connected(pu) = &ui_state.connection_state {
                                match pu.create_hub(hub) {
                                    Ok(hub) => {
                                        info!(
                                            "Connected to hub {}",
                                            hub.get_addr()
                                        );
                                        update_gui_selection =
                                            GuiSelection::ConnectedHub(
                                                *hub.get_addr(),
                                            );
                                        ui_state
                                            .connected_hubs
                                            .insert(*hub.get_addr(), hub);
                                    }
                                    Err(e) => {
                                        error!(
                                            "Error connecting to hub {}: {}",
                                            hub.addr, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            GuiSelection::ConnectedHub(addr) => {
                if let Some(hub) = ui_state.connected_hubs.get(&addr) {
                    // Show hub details and disconnect button
                    ui.label(hub.get_name());
                    ui.label(hub.get_type().to_string());
                    ui.label(hub.get_addr().to_string());

                    // Disconnect button
                    if ui.button("Disconnect").clicked() {
                        info!("Disconnecting from {}", hub.get_addr());
                        if let Err(e) = hub.disconnect() {
                            error!(
                                "Error disconnecting from hub {}: {}",
                                addr, e
                            );
                        }
                        ui_state.connected_hubs.remove(&addr);
                        ui_state.gui_selection = GuiSelection::None;
                    }
                } else {
                    ui.label(format!("Invalid selection: {}", addr));
                }
            }
        }

        if !update_gui_selection.is_none() {
            ui_state.gui_selection = update_gui_selection;
        }
    });
}
