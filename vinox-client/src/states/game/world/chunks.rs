use std::{collections::HashSet, time::Duration};

use bevy::{ecs::system::SystemParam, math::Vec3Swizzles, prelude::*};
use bevy_tweening::{lens::TransformPositionLens, *};
use vinox_common::world::chunks::{
    ecs::{CurrentChunks, RemoveChunk, SimulationRadius, ViewRadius},
    positions::{circle_points, ChunkPos},
    storage::{
        BlockData, BlockTable, ChunkData, RawChunk, VoxelVisibility, CHUNK_SIZE, CHUNK_SIZE_ARR,
        HORIZONTAL_DISTANCE, VERTICAL_DISTANCE,
    },
};

use crate::states::{
    components::GameState,
    game::rendering::meshing::{build_mesh, priority_mesh, NeedsMesh, PriorityMesh},
};

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Default, Resource)]
pub struct PlayerChunk {
    pub chunk_pos: IVec3,
}

#[derive(Default, Resource)]
pub struct PlayerBlock {
    pub pos: IVec3,
}

pub struct CreateChunkEvent {
    pub pos: IVec3,
    pub raw_chunk: RawChunk,
}

pub struct SetBlockEvent {
    pub chunk_pos: IVec3,
    pub voxel_pos: UVec3,
    pub block_type: BlockData,
}
pub struct UpdateChunkEvent {
    pub pos: IVec3,
}

#[derive(Default, Resource)]
pub struct ChunkQueue {
    pub mesh: Vec<(IVec3, RawChunk)>,
    pub remove: HashSet<IVec3>,
}

impl PlayerChunk {
    pub fn is_in_radius(&self, pos: IVec3, view_radius: &ViewRadius) -> bool {
        !(pos
            .xz()
            .as_vec2()
            .distance(self.chunk_pos.xz().as_vec2())
            .abs()
            .floor() as i32
            > view_radius.horizontal
            || (pos.y - self.chunk_pos.y).abs() > view_radius.vertical)
    }
}

pub fn update_player_location(
    player_query: Query<&Transform, With<ControlledPlayer>>,
    mut player_chunk: ResMut<PlayerChunk>,
    mut player_block: ResMut<PlayerBlock>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let new_chunk = ChunkPos::from_global_coords(
            player_transform.translation.x,
            player_transform.translation.y,
            player_transform.translation.z,
        );
        if new_chunk.as_ivec3() != player_chunk.chunk_pos {
            player_chunk.chunk_pos = new_chunk.as_ivec3();
        }
        if player_transform.translation.floor().as_ivec3() != player_block.pos {
            player_block.pos = player_transform.translation.floor().as_ivec3();
        }
    }
}

#[derive(SystemParam)]
pub struct ChunkManager<'w, 's> {
    // commands: Commands<'w, 's>,
    pub current_chunks: ResMut<'w, CurrentChunks>,
    // chunk_queue: ResMut<'w, ChunkQueue>,
    pub view_radius: Res<'w, ViewRadius>,
    pub chunk_query: Query<'w, 's, &'static ChunkData>,
}

impl<'w, 's> ChunkManager<'w, 's> {
    pub fn get_chunk_positions(&mut self, chunk_pos: ChunkPos) -> Vec<ChunkPos> {
        let mut chunks = Vec::new();

        for point in circle_points(&self.view_radius) {
            for y in -self.view_radius.vertical..=self.view_radius.vertical {
                let pos = chunk_pos.as_ivec3() + IVec3::new(point.x, y, point.y);
                chunks.push(ChunkPos {
                    x: pos.x as usize,
                    y: pos.y as usize,
                    z: pos.z as usize,
                });
            }
        }

        // chunks
        //     .sort_unstable_by_key(|key| (key.x - chunk_pos.x).abs() + (key.z - chunk_pos.z).abs());
        chunks
    }
    pub fn get_chunks_around_chunk(&mut self, pos: ChunkPos) -> Vec<&ChunkData> {
        let mut res = Vec::new();
        for chunk_pos in self.get_chunk_positions(pos).iter() {
            if let Some(entity) = self.current_chunks.get_entity(*chunk_pos) {
                if let Ok(chunk) = self.chunk_query.get(entity) {
                    res.push(chunk);
                }
            }
        }

        res
    }
    pub fn get_neighbors(&self, pos: ChunkPos) -> Option<Vec<ChunkData>> {
        let mut res = Vec::with_capacity(26);
        for chunk_entity in self.current_chunks.get_all_neighbors(pos) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                res.push(chunk.clone())
            }
        }
        if res.len() == 26 {
            Some(res)
        } else {
            None
        }
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    remove_chunks: Query<(&ChunkPos, Entity), With<RemoveChunk>>,
    mut current_chunks: ResMut<CurrentChunks>,
) {
    for (chunk, chunk_entity) in remove_chunks.iter() {
        if current_chunks.get_entity(*chunk).is_some() {
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    end: Vec3::new(
                        (chunk.x as i32 * (CHUNK_SIZE) as i32) as f32,
                        ((chunk.y as i32 * (CHUNK_SIZE) as i32) as f32) - CHUNK_SIZE as f32,
                        (chunk.z as i32 * (CHUNK_SIZE) as i32) as f32,
                    ),

                    start: Vec3::new(
                        (chunk.x as i32 * (CHUNK_SIZE) as i32) as f32,
                        (chunk.y as i32 * (CHUNK_SIZE) as i32) as f32,
                        (chunk.z as i32 * (CHUNK_SIZE) as i32) as f32,
                    ),
                },
            )
            .with_repeat_count(RepeatCount::Finite(1))
            .with_completed_event(0);
            commands.entity(chunk_entity).insert(Animator::new(tween));
            commands.entity(chunk_entity).remove::<RemoveChunk>();
            commands.entity(chunk_entity).remove::<ChunkData>();
            current_chunks.remove_entity(*chunk).unwrap();
        }
    }
}

pub fn destroy_chunks(mut commands: Commands, mut query_event: EventReader<TweenCompleted>) {
    for evt in query_event.iter() {
        if evt.user_data == 0 {
            commands.entity(evt.entity).despawn_recursive();
        }
    }
}

pub fn clear_unloaded_chunks(
    mut commands: Commands,
    chunks: Query<(&ChunkPos, Entity)>,
    player_chunk: Res<PlayerChunk>,
    view_radius: Res<ViewRadius>,
) {
    for (chunk, entity) in chunks.iter() {
        if player_chunk.is_in_radius(chunk.as_ivec3(), &view_radius) {
            continue;
        } else {
            commands.entity(entity).insert(RemoveChunk);
        }
    }
}

#[allow(clippy::nonminimal_bool)]
pub fn receive_chunks(
    mut current_chunks: ResMut<CurrentChunks>,
    mut commands: Commands,
    mut event: EventReader<CreateChunkEvent>,
    player_chunk: Res<PlayerChunk>,
    view_radius: Res<ViewRadius>,
    block_table: Res<BlockTable>,
) {
    for evt in event.iter() {
        if player_chunk.is_in_radius(evt.pos, &view_radius)
            && current_chunks
                .get_entity(ChunkPos::from_ivec3(evt.pos))
                .is_none()
        {
            let chunk_data = ChunkData::from_raw(evt.raw_chunk.clone());
            let chunk_id = commands
                .spawn(chunk_data.clone())
                .insert(ChunkPos::from_ivec3(evt.pos))
                .id();

            current_chunks.insert_entity(ChunkPos::from_ivec3(evt.pos), chunk_id);

            // Don't mark chunks that won't create any blocks
            if chunk_data.is_empty() {
                commands.entity(chunk_id).insert(NeedsMesh);
            }
        }
    }
}

pub fn set_block(
    mut commands: Commands,
    mut event: EventReader<SetBlockEvent>,
    current_chunks: Res<CurrentChunks>,
    mut chunks: Query<&mut ChunkData>,
) {
    for evt in event.iter() {
        if let Some(chunk_entity) = current_chunks.get_entity(ChunkPos::from_ivec3(evt.chunk_pos)) {
            if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                chunk.set(
                    evt.voxel_pos.x as usize,
                    evt.voxel_pos.y as usize,
                    evt.voxel_pos.z as usize,
                    evt.block_type.clone(),
                );

                match evt.voxel_pos.x {
                    0 => {
                        if let Some(neighbor_chunk) = current_chunks
                            .get_entity(ChunkPos::from_ivec3(evt.chunk_pos + IVec3::new(-1, 0, 0)))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    CHUNK_SIZE_ARR => {
                        if let Some(neighbor_chunk) = current_chunks
                            .get_entity(ChunkPos::from_ivec3(evt.chunk_pos + IVec3::new(1, 0, 0)))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    _ => {}
                }
                match evt.voxel_pos.y {
                    0 => {
                        if let Some(neighbor_chunk) = current_chunks
                            .get_entity(ChunkPos::from_ivec3(evt.chunk_pos + IVec3::new(0, -1, 0)))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    CHUNK_SIZE_ARR => {
                        if let Some(neighbor_chunk) = current_chunks
                            .get_entity(ChunkPos::from_ivec3(evt.chunk_pos + IVec3::new(0, 1, 0)))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    _ => {}
                }
                match evt.voxel_pos.z {
                    0 => {
                        if let Some(neighbor_chunk) = current_chunks
                            .get_entity(ChunkPos::from_ivec3(evt.chunk_pos + IVec3::new(0, 0, -1)))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    CHUNK_SIZE_ARR => {
                        if let Some(neighbor_chunk) = current_chunks
                            .get_entity(ChunkPos::from_ivec3(evt.chunk_pos + IVec3::new(0, 0, 1)))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    _ => {}
                }
            }
            commands.entity(chunk_entity).insert(PriorityMesh);
        }
    }
}

pub fn should_update_chunks(player_chunk: Res<PlayerChunk>) -> bool {
    player_chunk.is_changed()
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunks::default())
            .insert_resource(ChunkQueue::default())
            .insert_resource(PlayerChunk::default())
            .insert_resource(PlayerBlock::default())
            .insert_resource(ViewRadius {
                horizontal: HORIZONTAL_DISTANCE as i32,
                vertical: VERTICAL_DISTANCE as i32,
            })
            .insert_resource(SimulationRadius {
                horizontal: 4,
                vertical: 4,
            })
            .add_system(update_player_location.in_set(OnUpdate(GameState::Game)))
            .add_systems(
                (receive_chunks, set_block)
                    .chain()
                    .after(update_player_location)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                clear_unloaded_chunks
                    .after(receive_chunks)
                    .run_if(should_update_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                build_mesh
                    .after(clear_unloaded_chunks)
                    .in_set(OnUpdate(GameState::Game)),
            )
            .add_system(
                priority_mesh
                    .after(clear_unloaded_chunks)
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
