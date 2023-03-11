use bevy::prelude::*;

use crate::states::components::GameState;

pub fn start_loading(mut commands: Commands) {
    commands.insert_resource(NextState(Some(GameState::Loading)));
}
