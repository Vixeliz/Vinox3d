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

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
