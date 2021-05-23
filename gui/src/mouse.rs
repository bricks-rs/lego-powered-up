use crate::EntitySelected;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ElementState;
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
    mut hubs_query: Query<(Entity, &Sprite, &mut GlobalTransform)>,
    windows: Res<Windows>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut state: Local<ClickAndDragState>,
    mut selected_sender: EventWriter<EntitySelected>,
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
                if let Some((entity, _offset)) = state.selected_object {
                    selected_sender.send(EntitySelected(entity));
                }
            }
            ElementState::Released => {
                state.selected_object = None;
            }
            _ => {}
        }
    }

    // Doesn't matter whether the cursor moved or not - if an object is
    // selected then we update its position to match the cursor regardless
    if let Some((ent, offset)) = state.selected_object {
        //println!("mouse move ({}, {})", evt.delta.x, evt.delta.y);
        if let Ok((_, _, mut transform)) = hubs_query.get_mut(ent) {
            transform.translation =
                (cursor + offset).extend(transform.translation.z);
        }
    }
}

fn get_object(
    hubs_query: &mut Query<(Entity, &Sprite, &mut GlobalTransform)>,
    pos: Vec2,
) -> Option<(Entity, Vec2)> {
    for (entity, sprite, global_transform) in hubs_query.iter_mut() {
        let xy = global_transform.translation.truncate();
        if pos.cmpge(xy - sprite.size / 2.0).all()
            && pos.cmple(xy + sprite.size / 2.0).all()
        {
            let offset = xy - pos;
            return Some((entity, offset));
        }
    }
    None
}
