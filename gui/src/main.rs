use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use lego_powered_up::{BDAddr, PoweredUp};
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
        .run();
}

#[derive(Default)]
struct UiState {
    label: String,
    devices: Vec<DeviceInfo>,
    selected_device_idx: usize,
    available_hubs: HashSet<BDAddr>,
    connected_hubs: HashMap<BDAddr, HubInfo>,
    connection_state: ConnectionState,
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

fn ui(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    assets: Res<AssetServer>,
) {
    // Menu bar
    egui::TopPanel::top("Top panel").show(egui_ctx.ctx(), |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "File", |ui| {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            let mut selected_device_idx = ui_state.selected_device_idx;
            egui::ComboBox::from_label("").show_index(
                ui,
                &mut selected_device_idx,
                ui_state.devices.len(),
                |idx| ui_state.devices[idx].name.clone(),
            );
            ui_state.selected_device_idx = selected_device_idx;

            let mut do_disconnect = false;
            match &ui_state.connection_state {
                ConnectionState::NotConnected => {
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
                ConnectionState::Connected(_) => {
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
                    ui_state.connection_state = ConnectionState::NotConnected;
                }
            }
        });
    });

    egui::SidePanel::left("side_panel", 200.0).show(egui_ctx.ctx(), |ui| {
        ui.heading("Side panel");

        ui.horizontal(|ui| {
            ui.label("GAME");
        });
    });
}
