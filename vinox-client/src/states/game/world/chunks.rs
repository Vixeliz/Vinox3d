use std::{collections::HashSet, time::Duration};

use bevy::{ecs::system::SystemParam, prelude::*, utils::FloatOrd};
use bevy_tweening::{lens::TransformPositionLens, *};
use vinox_common::world::chunks::{
    ecs::{ChunkComp, ChunkPos, CurrentChunks, RemoveChunk, SimulationRadius, ViewRadius},
    positions::world_to_chunk,
    storage::{BlockData, RawChunk, CHUNK_SIZE, HORIZONTAL_DISTANCE, VERTICAL_DISTANCE},
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
        for x in -view_radius.horizontal..view_radius.horizontal {
            for z in -view_radius.horizontal..view_radius.horizontal {
                if x.pow(2) + z.pow(2) >= view_radius.horizontal.pow(2) {
                    continue;
                }
                let delta: IVec3 = (pos - self.chunk_pos).abs();
                return !(delta.x.pow(2) + delta.z.pow(2) > view_radius.horizontal.pow(2)
                    || delta.y > view_radius.vertical);
            }
        }
        false
    }
}

pub fn update_player_location(
    player_query: Query<&Transform, With<ControlledPlayer>>,
    mut player_chunk: ResMut<PlayerChunk>,
    mut player_block: ResMut<PlayerBlock>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let new_chunk = world_to_chunk(player_transform.translation);
        if new_chunk != player_chunk.chunk_pos {
            player_chunk.chunk_pos = new_chunk;
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
    pub chunk_query: Query<'w, 's, &'static ChunkComp>,
}

impl<'w, 's> ChunkManager<'w, 's> {
    pub fn get_chunk_positions(&mut self, chunk_pos: IVec3) -> Vec<IVec3> {
        let mut chunks = Vec::new();
        for x in -self.view_radius.horizontal..=self.view_radius.horizontal {
            for z in -self.view_radius.horizontal..=self.view_radius.horizontal {
                for y in -self.view_radius.vertical..=self.view_radius.vertical {
                    if x.pow(2) + z.pow(2) >= self.view_radius.horizontal.pow(2) {
                        continue;
                    }

                    let chunk_key = {
                        let pos: IVec3 = chunk_pos + IVec3::new(x, y, z);

                        pos
                    };
                    chunks.push(chunk_key);
                }
            }
        }
        chunks.sort_unstable_by_key(|key| FloatOrd(key.as_vec3().distance(chunk_pos.as_vec3())));
        chunks
    }
    pub fn get_chunks_around_chunk(&mut self, pos: IVec3) -> Vec<&ChunkComp> {
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
    pub fn get_neighbors(&self, pos: ChunkPos) -> Option<Vec<RawChunk>> {
        let mut res = Vec::with_capacity(26);
        for chunk_entity in self.current_chunks.get_all_neighbors(pos) {
            if let Ok(chunk) = self.chunk_query.get(chunk_entity) {
                res.push(chunk.chunk_data.clone())
            }
        }
        Some(res)
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    remove_chunks: Query<(&ChunkComp, Entity), With<RemoveChunk>>,
    mut current_chunks: ResMut<CurrentChunks>,
) {
    for (chunk, chunk_entity) in remove_chunks.iter() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            TransformPositionLens {
                end: Vec3::new(
                    (chunk.pos.0.x * (CHUNK_SIZE) as i32) as f32,
                    ((chunk.pos.0.y * (CHUNK_SIZE) as i32) as f32) - CHUNK_SIZE as f32,
                    (chunk.pos.0.z * (CHUNK_SIZE) as i32) as f32,
                ),

                start: Vec3::new(
                    (chunk.pos.0.x * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.0.y * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.0.z * (CHUNK_SIZE) as i32) as f32,
                ),
            },
        )
        .with_repeat_count(RepeatCount::Finite(1))
        .with_completed_event(0);
        commands.entity(chunk_entity).insert(Animator::new(tween));
        commands.entity(chunk_entity).remove::<RemoveChunk>();
        commands.entity(chunk_entity).remove::<ChunkComp>();
        current_chunks.remove_entity(chunk.pos.0).unwrap();
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
    chunks: Query<(&ChunkComp, Entity)>,
    player_chunk: Res<PlayerChunk>,
    view_radius: Res<ViewRadius>,
) {
    for (chunk, entity) in chunks.iter() {
        if player_chunk.is_in_radius(chunk.pos.0, &view_radius) {
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
) {
    for evt in event.iter() {
        if player_chunk.is_in_radius(evt.pos, &view_radius)
            && current_chunks.get_entity(evt.pos).is_none()
        {
            let chunk_id = commands
                .spawn(ChunkComp {
                    pos: ChunkPos(evt.pos),
                    chunk_data: evt.raw_chunk.to_owned(),
                    saved_entities: Vec::new(),
                    entities: Vec::new(),
                })
                .id();

            current_chunks.insert_entity(evt.pos, chunk_id);

            let mut empty = true;
            for block in evt.raw_chunk.palette.right_values() {
                let mut identifier = block.namespace.clone();
                identifier.push(':');
                identifier.push_str(&block.name);
                if identifier != "vinox:air" {
                    empty = false;
                }
            }

            if !empty {
                commands.entity(chunk_id).insert(NeedsMesh);
            }
        }
    }
}

pub fn set_block(
    mut commands: Commands,
    mut event: EventReader<SetBlockEvent>,
    current_chunks: Res<CurrentChunks>,
    mut chunks: Query<&mut ChunkComp>,
) {
    for evt in event.iter() {
        if let Some(chunk_entity) = current_chunks.get_entity(evt.chunk_pos) {
            if let Ok(mut chunk) = chunks.get_mut(chunk_entity) {
                chunk.chunk_data.add_block_state(&evt.block_type);
                chunk.chunk_data.set_block(evt.voxel_pos, &evt.block_type);

                match evt.voxel_pos.x {
                    1 => {
                        if let Some(neighbor_chunk) =
                            current_chunks.get_entity(evt.chunk_pos + IVec3::new(-1, 0, 0))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    CHUNK_SIZE => {
                        if let Some(neighbor_chunk) =
                            current_chunks.get_entity(evt.chunk_pos + IVec3::new(1, 0, 0))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    _ => {}
                }
                match evt.voxel_pos.y {
                    1 => {
                        if let Some(neighbor_chunk) =
                            current_chunks.get_entity(evt.chunk_pos + IVec3::new(0, -1, 0))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    CHUNK_SIZE => {
                        if let Some(neighbor_chunk) =
                            current_chunks.get_entity(evt.chunk_pos + IVec3::new(0, 1, 0))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    _ => {}
                }
                match evt.voxel_pos.z {
                    1 => {
                        if let Some(neighbor_chunk) =
                            current_chunks.get_entity(evt.chunk_pos + IVec3::new(0, 0, -1))
                        {
                            commands.entity(neighbor_chunk).insert(PriorityMesh);
                        }
                    }
                    CHUNK_SIZE => {
                        if let Some(neighbor_chunk) =
                            current_chunks.get_entity(evt.chunk_pos + IVec3::new(0, 0, 1))
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
                horizontal: HORIZONTAL_DISTANCE,
                vertical: VERTICAL_DISTANCE,
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
