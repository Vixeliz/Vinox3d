use bevy::{
    ecs::system::SystemParam,
    math::Vec3Swizzles,
    // utils::FloatOrd,
    prelude::*,
    tasks::AsyncComputeTaskPool,
};
use tokio::sync::mpsc::{Receiver, Sender};
use vinox_common::world::chunks::{
    ecs::{CurrentChunks, RemoveChunk, SimulationRadius, ViewRadius},
    positions::{circle_points, ChunkPos},
    storage::{ChunkData, HORIZONTAL_DISTANCE, VERTICAL_DISTANCE},
};

use crate::game::networking::components::SentChunks;

use super::{
    generation::generate_chunk,
    storage::{load_chunk, save_chunks, ChunksToSave, WorldDatabase, WorldInfo},
};

#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct LoadPoint(pub IVec3);

impl LoadPoint {
    pub fn is_in_radius(&self, pos: IVec3, view_radius: &ViewRadius) -> bool {
        !(pos
            .xz()
            .as_vec2()
            .distance(self.xz().as_vec2())
            .abs()
            .floor() as i32
            > view_radius.horizontal
            || (pos.y - self.y).abs() > view_radius.vertical)
    }
}

#[derive(Default, Resource, Debug)]
pub struct ChunkQueue {
    pub create: Vec<ChunkPos>,
    pub remove: Vec<ChunkPos>,
}

#[derive(SystemParam)]
pub struct ChunkManager<'w, 's> {
    // commands: Commands<'w, 's>,
    current_chunks: ResMut<'w, CurrentChunks>,
    // chunk_queue: ResMut<'w, ChunkQueue>,
    view_radius: Res<'w, ViewRadius>,
    chunk_query: Query<'w, 's, &'static ChunkData>,
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
    pub fn get_chunks_around_chunk(
        &mut self,
        pos: ChunkPos,
        sent_chunks: &SentChunks,
    ) -> Vec<(&ChunkData, ChunkPos)> {
        let mut res = Vec::new();
        for chunk_pos in self.get_chunk_positions(pos).iter() {
            if !sent_chunks.chunks.contains(chunk_pos) {
                if let Some(entity) = self.current_chunks.get_entity(*chunk_pos) {
                    if let Ok(chunk) = self.chunk_query.get(entity) {
                        res.push((chunk, *chunk_pos));
                    }
                }
            }
        }

        res
    }
}

pub fn generate_chunks_world(
    load_points: Query<&LoadPoint>,
    mut chunk_queue: ResMut<ChunkQueue>,
    mut commands: Commands,
    mut chunk_manager: ChunkManager,
    database: Res<WorldDatabase>,
) {
    for point in load_points.iter() {
        for pos in chunk_manager.get_chunk_positions(ChunkPos::from_ivec3(**point)) {
            if chunk_manager.current_chunks.get_entity(pos).is_none() {
                let data = database.connection.get().unwrap();
                if let Some(chunk) = load_chunk(pos, &data) {
                    let chunk_id = commands.spawn(ChunkData::from_raw(chunk)).insert(pos).id();
                    chunk_manager.current_chunks.insert_entity(pos, chunk_id);
                } else {
                    let chunk_id = commands.spawn_empty().id();
                    chunk_manager.current_chunks.insert_entity(pos, chunk_id);
                    chunk_queue.create.push(pos);
                }
            }
        }
    }
}

pub fn destroy_chunks(
    mut commands: Commands,
    mut current_chunks: ResMut<CurrentChunks>,
    remove_chunks: Query<&ChunkPos, With<RemoveChunk>>,
    mut load_points: Query<&mut SentChunks>,
) {
    for chunk in remove_chunks.iter() {
        for mut sent_chunks in load_points.iter_mut() {
            sent_chunks.chunks.remove(&*chunk);
        }
        commands
            .entity(current_chunks.remove_entity(*chunk).unwrap())
            .despawn_recursive();
    }
}

pub fn clear_unloaded_chunks(
    mut commands: Commands,
    chunks: Query<(&ChunkPos, Entity)>,
    load_points: Query<&LoadPoint>,
    view_radius: Res<ViewRadius>,
) {
    for (chunk, entity) in chunks.iter() {
        for load_point in load_points.iter() {
            if load_point.is_in_radius(chunk.as_ivec3(), &view_radius) {
                continue;
            } else {
                commands.entity(entity).insert(RemoveChunk);
            }
        }
    }
}

pub fn unsend_chunks(
    chunks: Query<&ChunkPos>,
    mut load_points: Query<(&LoadPoint, &mut SentChunks)>,
    view_radius: Res<ViewRadius>,
) {
    for (load_point, mut sent_chunks) in load_points.iter_mut() {
        for chunk in chunks.iter() {
            if !load_point.is_in_radius(chunk.as_ivec3(), &view_radius) {
                sent_chunks.chunks.remove(&*chunk);
            } else {
                continue;
            }
        }
    }
}

#[derive(Resource)]
pub struct ChunkChannel {
    pub tx: Sender<(ChunkData, ChunkPos)>,
    pub rx: Receiver<(ChunkData, ChunkPos)>,
}

impl Default for ChunkChannel {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(512);

        Self { tx, rx }
    }
}

pub fn process_save(mut chunks_to_save: ResMut<ChunksToSave>, database: Res<WorldDatabase>) {
    save_chunks(&chunks_to_save, &database.connection.get().unwrap());
    chunks_to_save.clear();
}

pub fn process_queue(
    mut commands: Commands,
    mut chunk_queue: ResMut<ChunkQueue>,
    mut chunk_channel: ResMut<ChunkChannel>,
    current_chunks: Res<CurrentChunks>,
    world_info: Res<WorldInfo>,
    mut chunks_to_save: ResMut<ChunksToSave>,
) {
    let cloned_seed = world_info.seed;
    let task_pool = AsyncComputeTaskPool::get();
    for chunk_pos in chunk_queue.create.drain(..) {
        let cloned_sender = chunk_channel.tx.clone();
        task_pool
            .spawn(async move {
                cloned_sender
                    .send((
                        ChunkData::from_raw(generate_chunk(chunk_pos.as_ivec3(), cloned_seed)),
                        chunk_pos,
                    ))
                    .await
                    .ok();
            })
            .detach();
    }
    chunk_queue.create.clear();
    while let Ok(chunk) = chunk_channel.rx.try_recv() {
        let chunk_pos = chunk.1.clone();

        chunks_to_save.push((chunk_pos, chunk.0.to_raw()));
        commands
            .entity(current_chunks.get_entity(chunk_pos).unwrap())
            .insert(chunk);
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunksToSave::default())
            .insert_resource(CurrentChunks::default())
            .insert_resource(ChunkQueue::default())
            .insert_resource(ViewRadius {
                horizontal: HORIZONTAL_DISTANCE as i32,
                vertical: VERTICAL_DISTANCE as i32,
            })
            .insert_resource(SimulationRadius {
                vertical: 4,
                horizontal: 4,
            })
            .add_systems((clear_unloaded_chunks, unsend_chunks, generate_chunks_world))
            .add_system(process_queue.after(clear_unloaded_chunks))
            .add_system(process_save.after(process_queue))
            .add_startup_system(|mut commands: Commands| {
                commands.insert_resource(ChunkChannel::default());
            })
            .add_system(destroy_chunks.after(process_queue));
    }
}
