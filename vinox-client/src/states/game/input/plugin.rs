use bevy::prelude::*;

use crate::states::components::GameState;

use super::player::{
    collision_movement_system, interact, movement_input, spawn_camera, update_aabb,
    MouseSensitivity,
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
                update_aabb,
            )
                .in_set(OnUpdate(GameState::Game)),
        );
    }
}
