use leafwing_input_manager::prelude::*;

use bevy::prelude::*;

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

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

#[derive(Resource, Clone, Debug)]
pub struct GameOptions {
    pub input: InputMap<GameActions>,
    pub fov: f32,
    pub dark_theme: bool,
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
            fov: 4.0,
            dark_theme: false,
        }
    }
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
