use crate::HubInfo;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ElementState;
use bevy::math::XYZ;
use bevy::prelude::*;

#[derive(Default)]
pub struct ClickAndDragState {
    // is Some(_) if clicked state on an object, otherwise None.
    // An extra bool to track whether is_clicked or not is not needed
    // because the Option encodes this
    // The vec2 is the cursor offset from the origin of the shape
    selected_object: Option<(Entity, Vec2)>,
}

pub fn update(
    mut hubs_query: Query<(Entity, &HubInfo, &Sprite, &mut GlobalTransform)>,
    windows: Res<Windows>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut motion_evr: EventReader<MouseMotion>,
    mut state: Local<ClickAndDragState>,
) {
    let window = windows.get_primary().unwrap();
    // cursor has origin at bottom left
    let cursor = window.cursor_position().unwrap_or_default();
    let win_size = Vec2::new(window.width(), window.height());
    // translate by adding half the window dimensions and inverting the axes
    let cursor = (win_size / 2.0 - cursor) * Vec2::new(-1.0, -1.0);

    for evt in mousebtn_evr.iter() {
        match evt.state {
            ElementState::Pressed if evt.button == MouseButton::Left => {
                // Grab the hub that is underneath the cursor (if any)
                state.selected_object = get_object(&mut hubs_query, cursor);
            }
            ElementState::Released => {
                state.selected_object = None;
            }
            _ => {}
        }
    }

    for evt in motion_evr.iter() {
        if let Some((ent, offset)) = state.selected_object {
            //println!("mouse move ({}, {})", evt.delta.x, evt.delta.y);
            if let Ok((_, _, _, mut transform)) = hubs_query.get_mut(ent) {
                transform.translation =
                    (cursor + offset).extend(transform.translation.z);
            }
        }
    }
}

fn get_object(
    hubs_query: &mut Query<(Entity, &HubInfo, &Sprite, &mut GlobalTransform)>,
    pos: Vec2,
) -> Option<(Entity, Vec2)> {
    for (entity, hub, sprite, global_transform) in hubs_query.iter_mut() {
        let xy = global_transform.translation.truncate();
        println!("pos: {:?}, entity: {:?},  size: {:?}", pos, xy, sprite.size);
        if pos.cmpge(xy - sprite.size / 2.0).all()
            && pos.cmple(xy + sprite.size / 2.0).all()
        {
            println!("Clicked a thing! {:?}", entity);
            let offset = xy - pos;
            return Some((entity, offset));
        }
    }
    None
}
