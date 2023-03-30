use std::collections::HashSet;
use tokio::sync::mpsc::{Receiver, Sender};

use bevy::{prelude::*, render::primitives::Aabb, tasks::AsyncComputeTaskPool, utils::FloatOrd};
use bevy_tweening::*;
use vinox_common::world::chunks::{
    ecs::{
        update_chunk_lights, update_priority_chunk_lights, ChunkCell, ChunkManager, ChunkUpdate,
        CurrentChunks, LoadPoint, NeedsChunkData, RemoveChunk, SimulationRadius,
    },
    positions::{ChunkPos, RelativeVoxelPos, VoxelPos},
    storage::{BlockData, BlockTable, ChunkData, RawChunk, HORIZONTAL_DISTANCE, VERTICAL_DISTANCE},
};

use crate::states::{
    components::GameState,
    game::rendering::meshing::{build_mesh, priority_mesh},
};

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Default, Resource)]
pub struct PlayerChunk {
    pub chunk_pos: ChunkPos,
}

#[derive(Default, Resource)]
pub struct PlayerBlock {
    pub pos: VoxelPos,
}

#[derive(Default, Resource, Debug)]
pub struct PlayerTargetedBlock {
    pub block: Option<BlockData>,
    pub pos: Option<VoxelPos>,
}

#[derive(Default, Debug)]
pub enum VoxelAxis {
    #[default]
    North,
    South,
    West,
    East,
}

#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub struct PlayerDirection(pub VoxelAxis);

pub struct CreateChunkEvent {
    pub pos: ChunkPos,
    pub raw_chunk: RawChunk,
}

pub struct SetBlockEvent {
    pub chunk_pos: ChunkPos,
    pub voxel_pos: RelativeVoxelPos,
    pub block_type: BlockData,
}
pub struct UpdateChunkEvent {
    pub pos: ChunkPos,
}

pub fn update_player_location(
    player_query: Query<&Aabb, With<ControlledPlayer>>,
    mut player_chunk: ResMut<PlayerChunk>,
    mut player_block: ResMut<PlayerBlock>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let new_chunk = ChunkPos::from_world(VoxelPos::from_world(player_transform.center.into()));
        if new_chunk != player_chunk.chunk_pos {
            player_chunk.chunk_pos = new_chunk;
        }
        if VoxelPos::from_world(player_transform.center.into()) != player_block.pos {
            player_block.pos = VoxelPos::from_world(player_transform.center.into());
        }
    }
}
pub fn update_player_direction(
    mut player_direction: ResMut<PlayerDirection>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
) {
    if let Ok(camera) = camera_query.get_single() {
        let forward = camera.forward();

        let east_dot = forward.dot(Vec3::X);
        let west_dot = forward.dot(Vec3::NEG_X);
        let north_dot = forward.dot(Vec3::Z);
        let south_dot = forward.dot(Vec3::NEG_Z);
        let numbers = [east_dot, west_dot, north_dot, south_dot];
        let closest = numbers.iter().max_by_key(|&num| FloatOrd(*num)).unwrap();
        **player_direction = if *closest == east_dot {
            VoxelAxis::East
        } else if *closest == west_dot {
            VoxelAxis::West
        } else if *closest == north_dot {
            VoxelAxis::North
        } else {
            VoxelAxis::South
        };
    }
}
pub fn unload_chunks(
    mut commands: Commands,
    remove_chunks: Query<(&ChunkPos, Entity), With<RemoveChunk>>,
    mut current_chunks: ResMut<CurrentChunks>,
) {
    for (chunk, entity) in remove_chunks.iter() {
        current_chunks.remove_entity(*chunk).ok_or(0).ok();
        commands.entity(entity).despawn_recursive();
    }
}

pub fn destroy_chunks(mut commands: Commands, mut query_event: EventReader<TweenCompleted>) {
    for evt in query_event.iter() {
        if evt.user_data == 0 {
            commands.entity(evt.entity).despawn_recursive();
        }
    }
}

#[allow(clippy::nonminimal_bool)]
pub fn receive_chunks(
    mut current_chunks: ResMut<CurrentChunks>,
    mut commands: Commands,
    mut event: EventReader<CreateChunkEvent>,
    // player_chunk: Res<PlayerChunk>,
    has_data: Query<With<NeedsChunkData>>,
    load_point: Query<&LoadPoint, With<ControlledPlayer>>,
    block_table: Res<BlockTable>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    if let Ok(load_point) = load_point.get_single() {
        for evt in event.iter() {
            if load_point.is_in_radius(&evt.pos) {
                if let Some(chunk_entity) = current_chunks.get_entity(evt.pos) {
                    if has_data.get(chunk_entity).is_ok() {
                        let mut chunk = ChunkData::from_raw(evt.raw_chunk.clone());
                        if !chunk.is_empty(&block_table) {
                            commands.entity(chunk_entity).insert(ChunkUpdate);
                        }
                        commands.entity(chunk_entity).insert(chunk);
                        commands.entity(chunk_entity).remove::<NeedsChunkData>();
                    }
                }
            }
        }
    }
}

pub fn set_block(mut event: EventReader<SetBlockEvent>, mut chunk_manager: ChunkManager) {
    for evt in event.iter() {
        chunk_manager.set_block(
            VoxelPos::from_offsets(evt.voxel_pos, evt.chunk_pos),
            evt.block_type.clone(),
        );
    }
}

pub fn should_update_chunks(player_chunk: Res<PlayerChunk>) -> bool {
    player_chunk.is_changed()
}

pub struct ChunkPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ChunkSet {
    UpdateChunks,
    ReceiveChunks,
    UnloadChunks,
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunks::default())
            .insert_resource(PlayerChunk::default())
            .insert_resource(PlayerBlock::default())
            .insert_resource(PlayerDirection::default())
            .insert_resource(PlayerTargetedBlock::default())
            .insert_resource(SimulationRadius {
                horizontal: 4,
                vertical: 4,
            })
            .add_system(update_player_location.in_set(OnUpdate(GameState::Game)))
            .add_system(update_player_direction.in_set(OnUpdate(GameState::Game)))
            .add_systems(
                (receive_chunks, set_block)
                    .chain()
                    .after(update_player_location)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                update_chunk_lights
                    .after(receive_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                update_priority_chunk_lights
                    .after(receive_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                build_mesh
                    .after(update_chunk_lights)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                priority_mesh
                    .after(update_chunk_lights)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                unload_chunks
                    .after(build_mesh)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                destroy_chunks
                    .after(unload_chunks)
                    // .after(build_mesh)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_event::<UpdateChunkEvent>()
            .add_event::<SetBlockEvent>()
            .add_event::<CreateChunkEvent>();
    }
}
