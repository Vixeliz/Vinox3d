use bevy::prelude::*;

use super::start::{new_server, setup_loadables};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_loadables)
            .add_startup_system(new_server);
    }
}
