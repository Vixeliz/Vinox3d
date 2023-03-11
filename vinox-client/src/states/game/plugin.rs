use crate::states::components::{despawn_with, Game, GameState};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(despawn_with::<Game>.in_schedule(OnExit(GameState::Game)));
    }
}
