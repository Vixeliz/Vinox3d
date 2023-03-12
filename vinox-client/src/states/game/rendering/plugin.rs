use bevy::prelude::*;
use bevy_spectator::SpectatorPlugin;
use vinox_common::networking::protocol::EntityBuffer;

use crate::states::components::GameState;

use super::meshing::{
    create_chunk_material, process_queue, process_task, sort_chunks, sort_faces, ChunkMaterial,
    MeshQueue, SortFaces,
};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SpectatorPlugin)
            .insert_resource(MeshQueue::default())
            .insert_resource(ChunkMaterial::default())
            .add_system(create_chunk_material.in_schedule(OnEnter(GameState::Game)))
            .add_systems(
                (process_queue, process_task, sort_faces, sort_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1.0,
            })
            .add_event::<SortFaces>();
    }
}
