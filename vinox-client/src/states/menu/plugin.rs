use std::env;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use vinox_common::networking::protocol::NetworkIP;

use crate::states::components::{despawn_with, GameState, Menu};

use super::ui::{
    configure_visuals, create_ui, options, save_options, start, ui_events, update_ui_scale_factor,
    InOptions,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let args: Vec<String> = env::args().collect();

        let mut ip = "127.0.0.1".to_string();
        if let Some(idx) = args.iter().position(|i| i == "--address") {
            if idx + 1 < args.len() && !args[idx + 1].starts_with("--") {
                ip = args[idx + 1].to_string();
                app.world
                    .insert_resource(NextState(Some(GameState::Loading)));
            }
        }

        app.add_plugin(EguiPlugin)
            .insert_resource(InOptions(false))
            .insert_resource(NetworkIP(ip))
            .add_systems(
                (
                    save_options,
                    create_ui,
                    options,
                    ui_events,
                    configure_visuals,
                    update_ui_scale_factor,
                )
                    .chain()
                    .in_set(OnUpdate(GameState::Menu)),
            )
            .add_system(start.in_schedule(OnEnter(GameState::Menu)))
            .add_system(despawn_with::<Menu>.in_schedule(OnExit(GameState::Menu)));
    }
}
