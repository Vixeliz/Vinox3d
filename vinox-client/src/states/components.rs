use ron::ser::{to_string_pretty, PrettyConfig};
use std::io::Write;
use std::{fs::File, path::PathBuf};

use leafwing_input_manager::prelude::*;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Deref, DerefMut)]
pub struct ProjectPath(pub PathBuf);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    Loading,
    #[default]
    Menu,
    Game,
}

#[derive(Default, Component, Clone)]
pub struct Menu;
#[derive(Default, Component, Clone)]
pub struct Game;
#[derive(Default, Component, Clone)]
pub struct Loading;

#[derive(
    Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord,
)]
pub enum GameActions {
    Forward,
    Backward,
    Right,
    Left,
    Chat,
    Jump,
    PrimaryInteract,
    SecondaryInteract,
    Run,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GameOptions {
    pub input: InputMap<GameActions>,
    pub fov: f32,
    pub dark_theme: bool,
    pub user_name: String,
    pub standard_bar: bool,
}

impl Default for GameOptions {
    fn default() -> GameOptions {
        let mut input = InputMap::new([
            (KeyCode::W, GameActions::Forward),
            (KeyCode::S, GameActions::Backward),
            (KeyCode::D, GameActions::Right),
            (KeyCode::A, GameActions::Left),
            (KeyCode::T, GameActions::Chat),
            (KeyCode::Space, GameActions::Jump),
            (KeyCode::LShift, GameActions::Run),
        ]);

        input.insert(MouseButton::Left, GameActions::PrimaryInteract);
        input.insert(MouseButton::Right, GameActions::SecondaryInteract);

        GameOptions {
            input,
            fov: 70.0,
            dark_theme: true,
            user_name: "User".to_string(),
            standard_bar: false,
        }
    }
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn save_game_options(options: GameOptions, path: PathBuf) {
    let final_path = path.join("config.ron");
    if let Ok(mut output) = File::create(final_path) {
        let pretty = PrettyConfig::new()
            .depth_limit(2)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let s = to_string_pretty(&options, pretty).ok().unwrap();
        write!(output, "{s}").ok();
    }
}
