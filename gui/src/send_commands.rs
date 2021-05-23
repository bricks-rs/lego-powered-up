use crate::{HubInfo, UiState};
use bevy::prelude::*;
use lego_powered_up::{hubs::Port, notifications::Power, BDAddr};
use std::collections::HashMap;

pub type HubCache = HashMap<BDAddr, [i8; 4]>;

pub(crate) fn send_commands(
    hub_objects_query: Query<&HubInfo>,
    mut hub_cache: Local<HubCache>,
    ui_state: Res<UiState>,
) {
    for hub in hub_objects_query.iter() {
        if let Some(addr) = hub.addr {
            // the hub object has an assigned address
            let mut changed = [true; 4];
            if let Some(cache) = hub_cache.get_mut(&addr) {
                // have a cached value - check whether it has changed
                // if no change then don't send an update
                for idx in 0..4 {
                    changed[idx] = hub.motor_speeds[idx] != cache[idx];
                }
            }
            hub_cache.insert(addr, hub.motor_speeds);

            if let Some(controller) = ui_state.connected_hubs.get(&addr) {
                if changed[0] {
                    let mut motor = controller.port(Port::A).unwrap();
                    if let Err(e) = motor.start_speed(
                        hub.motor_speeds[0],
                        Power::from_i8(hub.motor_speeds[0]).unwrap(),
                    ) {
                        error!("Error starting motor: {}", e);
                    }
                }

                if changed[1] {
                    let mut motor = controller.port(Port::B).unwrap();
                    if let Err(e) = motor.start_speed(
                        hub.motor_speeds[1],
                        Power::from_i8(hub.motor_speeds[1]).unwrap(),
                    ) {
                        error!("Error starting motor: {}", e);
                    }
                }

                if changed[2] {
                    let mut motor = controller.port(Port::C).unwrap();
                    if let Err(e) = motor.start_speed(
                        hub.motor_speeds[2],
                        Power::from_i8(hub.motor_speeds[2]).unwrap(),
                    ) {
                        error!("Error starting motor: {}", e);
                    }
                }

                if changed[3] {
                    let mut motor = controller.port(Port::D).unwrap();
                    if let Err(e) = motor.start_speed(
                        hub.motor_speeds[3],
                        Power::from_i8(hub.motor_speeds[3]).unwrap(),
                    ) {
                        error!("Error starting motor: {}", e);
                    }
                }
            }
        }
    }
}
