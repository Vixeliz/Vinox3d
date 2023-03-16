use std::env;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use vinox_common::networking::protocol::NetworkIP;

use crate::states::{
    components::{despawn_with, GameState, Menu},
    game::networking::components::UserName,
};

use super::ui::{
    configure_visuals, create_ui, options, start, ui_events, update_ui_scale_factor, InOptions,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let args: Vec<String> = env::args().collect();

        let mut ip = "127.0.0.1".to_string();
        match args.len() {
            1 => {}
            2 => {
                ip = args[1].to_string();
            }
            _ => {}
        }

        app.add_plugin(EguiPlugin)
            .insert_resource(InOptions(false))
            .insert_resource(NetworkIP(ip))
            .insert_resource(UserName("User".to_string()))
            .add_systems(
                (
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
