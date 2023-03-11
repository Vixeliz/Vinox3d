use std::env;

use bevy::prelude::*;
use vinox_common::networking::protocol::NetworkIP;

use crate::states::components::{despawn_with, GameState, Menu};

use super::ui::start_loading;

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

        app.insert_resource(NetworkIP(ip))
            .add_system(start_loading.in_schedule(OnEnter(GameState::Menu)))
            .add_system(despawn_with::<Menu>.in_schedule(OnExit(GameState::Menu)));
    }
}
