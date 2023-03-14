use bevy::{
    ecs::system::SystemParam,
    math::Vec3Swizzles,
    // utils::FloatOrd,
    prelude::*,
    tasks::AsyncComputeTaskPool,
};
use rand::Rng;
use tokio::sync::mpsc::{Receiver, Sender};
use vinox_common::world::chunks::{
    ecs::{ChunkComp, ChunkPos, CurrentChunks, RemoveChunk, SimulationRadius, ViewRadius},
    storage::{HORIZONTAL_DISTANCE, VERTICAL_DISTANCE},
};

use crate::game::networking::components::SentChunks;

use super::{
    generation::generate_chunk,
    storage::{insert_chunk, load_chunk, WorldDatabase},
};

#[derive(Resource, Default)]
pub struct WorldSeed(pub u32);

#[derive(Component, Default, Clone)]
pub struct LoadPoint(pub IVec3);

impl LoadPoint {
    pub fn is_in_radius(&self, pos: IVec3, view_radius: &ViewRadius) -> bool {
        !(pos
            .xz()
            .as_vec2()
            .distance(self.0.xz().as_vec2())
            .abs()
            .floor() as i32
            > view_radius.horizontal
            || (pos.y - self.0.y).abs() > view_radius.vertical)
    }
}

#[derive(Default, Resource, Debug)]
pub struct ChunkQueue {
    pub create: Vec<IVec3>,
    pub remove: Vec<IVec3>,
}

#[derive(SystemParam)]
pub struct ChunkManager<'w, 's> {
    // commands: Commands<'w, 's>,
    current_chunks: ResMut<'w, CurrentChunks>,
    // chunk_queue: ResMut<'w, ChunkQueue>,
    view_radius: Res<'w, ViewRadius>,
    chunk_query: Query<'w, 's, &'static ChunkComp>,
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
        chunks
    }
    pub fn get_chunks_around_chunk(
        &mut self,
        pos: IVec3,
        sent_chunks: &SentChunks,
    ) -> Vec<&ChunkComp> {
        let mut res = Vec::new();
        for chunk_pos in self.get_chunk_positions(pos).iter() {
            if !sent_chunks.chunks.contains(chunk_pos) {
                if let Some(entity) = self.current_chunks.get_entity(*chunk_pos) {
                    if let Ok(chunk) = self.chunk_query.get(entity) {
                        res.push(chunk);
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
        for pos in chunk_manager.get_chunk_positions(point.0) {
            if chunk_manager.current_chunks.get_entity(pos).is_none() {
                let data = database.connection.lock().unwrap();
                if let Some(chunk) = load_chunk(pos, &data) {
                    let chunk_id = commands
                        .spawn(ChunkComp {
                            pos: ChunkPos(pos),
                            chunk_data: chunk,
                            entities: Vec::new(),
                            saved_entities: Vec::new(),
                        })
                        .id();
                    chunk_manager.current_chunks.insert_entity(pos, chunk_id);
                } else {
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
            sent_chunks.chunks.remove(&chunk.0);
        }
        commands
            .entity(current_chunks.remove_entity(chunk.0).unwrap())
            .despawn_recursive();
    }
}

pub fn clear_unloaded_chunks(
    mut commands: Commands,
    chunks: Query<(&ChunkComp, Entity)>,
    load_points: Query<&LoadPoint>,
    view_radius: Res<ViewRadius>,
) {
    for (chunk, entity) in chunks.iter() {
        for load_point in load_points.iter() {
            if load_point.is_in_radius(chunk.pos.0, &view_radius) {
                continue;
            } else {
                commands.entity(entity).insert(RemoveChunk);
            }
        }
    }
}

pub fn unsend_chunks(
    chunks: Query<&ChunkComp>,
    mut load_points: Query<(&LoadPoint, &mut SentChunks)>,
    view_radius: Res<ViewRadius>,
) {
    for (load_point, mut sent_chunks) in load_points.iter_mut() {
        for chunk in chunks.iter() {
            if !load_point.is_in_radius(chunk.pos.0, &view_radius) {
                sent_chunks.chunks.remove(&chunk.pos.0);
            } else {
                continue;
            }
        }
    }
}

#[derive(Resource)]
pub struct ChunkChannel(pub (Sender<ChunkComp>, Receiver<ChunkComp>));

pub fn process_queue(
    mut commands: Commands,
    mut chunk_queue: ResMut<ChunkQueue>,
    mut chunk_channel: ResMut<ChunkChannel>,
    mut current_chunks: ResMut<CurrentChunks>,
    seed: Res<WorldSeed>,
    database: Res<WorldDatabase>,
) {
    let cloned_seed = seed.0;
    let task_pool = AsyncComputeTaskPool::get();
    for chunk_pos in chunk_queue.create.drain(..) {
        let cloned_sender = chunk_channel.0 .0.clone();
        task_pool
            .spawn(async move {
                cloned_sender
                    .send(ChunkComp {
                        pos: ChunkPos(chunk_pos),
                        chunk_data: generate_chunk(chunk_pos, cloned_seed),
                        entities: Vec::new(),
                        saved_entities: Vec::new(),
                    })
                    .await
                    .ok();
            })
            .detach();
    }
    while let Ok(chunk) = chunk_channel.0 .1.try_recv() {
        let chunk_pos = chunk.pos.0;
        let data = database.connection.lock().unwrap();
        insert_chunk(chunk.pos.0, &chunk.chunk_data, &data);
        let chunk_id = commands.spawn(chunk).id();
        current_chunks.insert_entity(chunk_pos, chunk_id);
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunks::default())
            .insert_resource(ChunkQueue::default())
            .insert_resource(ViewRadius {
                horizontal: HORIZONTAL_DISTANCE,
                vertical: VERTICAL_DISTANCE,
            })
            .insert_resource(SimulationRadius {
                vertical: 4,
                horizontal: 4,
            })
            .insert_resource(WorldSeed(rand::thread_rng().gen_range(0..u32::MAX)))
            .add_systems((clear_unloaded_chunks, unsend_chunks, generate_chunks_world))
            .add_system(process_queue.after(clear_unloaded_chunks))
            .add_startup_system(|mut commands: Commands| {
                commands.insert_resource(ChunkChannel(tokio::sync::mpsc::channel(512)));
            })
            .add_system(destroy_chunks.after(process_queue));
    }
}
