use bevy::prelude::*;

use crate::states::components::GameState;

use super::player::{
    cursor_grab_system, debug_input, handle_movement, interact, spawn_camera, ui_input, update_fov,
    update_input, update_visual_position, update_vsync, MouseSensitivity,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseSensitivity(1.0)).add_systems(
            (
                spawn_camera,
                handle_movement,
                interact,
                update_visual_position,
                cursor_grab_system.after(interact),
                update_fov,
                update_input,
                update_vsync,
                ui_input,
                debug_input,
            )
                .in_set(OnUpdate(GameState::Game)),
        );
    }
}
