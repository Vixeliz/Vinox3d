use crate::states::components::{despawn_with, Game, GameActions, GameState};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use vinox_common::world::chunks::light::LightPlugin;

use super::{
    input::plugin::InputPlugin, networking::plugin::NetworkingPlugin,
    rendering::plugin::RenderingPlugin, ui::plugin::UiPlugin, world::chunks::ChunkPlugin,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<GameActions>::default())
            .add_plugin(RenderingPlugin)
            .add_plugin(ChunkPlugin)
            .add_plugin(NetworkingPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(UiPlugin)
            .add_plugin(LightPlugin)
            // .add_plugin(LogDiagnosticsPlugin::default())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_system(despawn_with::<Game>.in_schedule(OnExit(GameState::Game)));
    }
}
