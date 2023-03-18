use crate::states::components::GameState;

use super::{
    dropdown::{create_ui, ConsoleOpen, Toast},
    inventory::{inventory, status_bar},
};
use bevy::prelude::*;

pub struct UiPlugin;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InUi(pub bool);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConsoleOpen(false))
            .insert_resource(InUi(false))
            .insert_resource(Toast::default())
            .add_systems(
                (create_ui, status_bar, inventory)
                    .chain()
                    .in_set(OnUpdate(GameState::Game)),
            );
    }
}
