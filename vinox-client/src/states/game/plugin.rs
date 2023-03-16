use crate::states::components::{despawn_with, Game, GameState};
use bevy::prelude::*;

use super::{
    input::plugin::InputPlugin, networking::plugin::NetworkingPlugin,
    rendering::plugin::RenderingPlugin, ui::plugin::UiPlugin, world::chunks::ChunkPlugin,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenderingPlugin)
            .add_plugin(ChunkPlugin)
            .add_plugin(NetworkingPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(UiPlugin)
            .add_system(despawn_with::<Game>.in_schedule(OnExit(GameState::Game)));
    }
}
