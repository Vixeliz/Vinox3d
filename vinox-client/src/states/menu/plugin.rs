use bevy::prelude::*;

use crate::states::components::{despawn_with, GameState, Menu};

use super::ui::start_loading;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_loading.in_schedule(OnEnter(GameState::Menu)))
            .add_system(despawn_with::<Menu>.in_schedule(OnExit(GameState::Menu)));
    }
}
