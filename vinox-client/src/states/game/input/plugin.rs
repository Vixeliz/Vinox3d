use bevy::prelude::*;

use crate::states::components::GameState;

use super::player::{
    collision_movement_system, cursor_grab_system, interact, movement_input, spawn_camera,
    ui_input, update_aabb, update_fov, update_input, update_vsync, MouseSensitivity,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseSensitivity(1.0)).add_systems(
            (
                spawn_camera,
                movement_input,
                interact,
                collision_movement_system,
                cursor_grab_system.after(interact),
                update_aabb,
                update_fov,
                update_input,
                update_vsync,
                ui_input,
            )
                .in_set(OnUpdate(GameState::Game)),
        );
    }
}
