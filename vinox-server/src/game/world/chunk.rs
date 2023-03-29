use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use tokio::sync::mpsc::{Receiver, Sender};
use vinox_common::world::chunks::{
    ecs::{ChunkManager, CurrentChunks, RemoveChunk, SentChunks, SimulationRadius, ViewRadius},
    positions::ChunkPos,
    storage::{BlockTable, ChunkData, HORIZONTAL_DISTANCE, VERTICAL_DISTANCE},
};

use crate::game::networking::components::SaveGame;

use super::{
    generation::{generate_chunk, ToBePlaced},
    storage::{load_chunk, save_chunks, ChunksToSave, WorldDatabase, WorldInfo},
};

#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct LoadPoint(pub IVec3);

impl LoadPoint {
    pub fn is_in_radius(&self, pos: IVec3, view_radius: &ViewRadius) -> bool {
        !(pos.x > (view_radius.horizontal + self.0.x)
            || pos.x < (-view_radius.horizontal + self.0.x)
            || pos.z > (view_radius.horizontal + self.0.z)
            || pos.z < (-view_radius.horizontal + self.0.z)
            || pos.y > (view_radius.vertical + self.0.y)
            || pos.y < (-view_radius.vertical + self.0.y))
    }
}

#[derive(Default, Resource, Debug)]
pub struct ChunkQueue {
    pub create: Vec<ChunkPos>,
    pub remove: Vec<ChunkPos>,
}

pub fn generate_chunks_world(
    load_points: Query<&LoadPoint>,
    mut chunk_queue: ResMut<ChunkQueue>,
    mut commands: Commands,
    mut chunk_manager: ChunkManager,
    database: Res<WorldDatabase>,
    save: Res<SaveGame>,
) {
    for point in load_points.iter() {
        for pos in chunk_manager.get_chunk_positions(ChunkPos(**point)) {
            if chunk_manager.current_chunks.get_entity(pos).is_none() {
                let data = database.connection.get().unwrap();
                if let Some(chunk) = load_chunk(pos, &data) {
                    if **save {
                        let chunk_id = commands.spawn(ChunkData::from_raw(chunk)).insert(pos).id();
                        chunk_manager.current_chunks.insert_entity(pos, chunk_id);
                        continue;
                    }
                }
                let chunk_id = commands.spawn(pos).id();
                chunk_manager.current_chunks.insert_entity(pos, chunk_id);
                chunk_queue.create.push(pos);
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
            sent_chunks.chunks.remove(chunk);
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
            if load_point.is_in_radius(**chunk, &view_radius) {
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
            if !load_point.is_in_radius(**chunk, &view_radius) {
                sent_chunks.chunks.remove(chunk);
            } else {
                continue;
            }
        }
    }
}

// #[derive(Resource)]
// pub struct ChunkChannel {
//     pub tx: Sender<(ChunkData, ChunkPos)>,
//     pub rx: Receiver<(ChunkData, ChunkPos)>,
// }

// impl Default for ChunkChannel {
//     fn default() -> Self {
//         let (tx, rx) = tokio::sync::mpsc::channel(512);

//         Self { tx, rx }
//     }
// }

pub fn process_save(mut chunks_to_save: ResMut<ChunksToSave>, database: Res<WorldDatabase>) {
    save_chunks(&chunks_to_save, &database.connection.get().unwrap());
    chunks_to_save.clear();
}

#[derive(Component)]
pub struct GenTask(Task<(ChunkData, ChunkPos)>);

pub fn process_queue(
    mut commands: Commands,
    mut chunk_queue: ResMut<ChunkQueue>,
    mut gen_task: Query<(Entity, &mut GenTask)>,
    // mut chunk_channel: ResMut<ChunkChannel>,
    current_chunks: Res<CurrentChunks>,
    world_info: Res<WorldInfo>,
    mut chunks_to_save: ResMut<ChunksToSave>,
    mut to_be_placed: ResMut<ToBePlaced>,
    block_table: Res<BlockTable>,
    save: Res<SaveGame>,
) {
    let cloned_seed = world_info.seed;
    let task_pool = AsyncComputeTaskPool::get();
    for chunk_pos in chunk_queue.create.drain(..) {
        let cloned_table = block_table.clone();
        // let cloned_place = to_be_placed.clone();
        let task = task_pool.spawn(async move {
            (
                ChunkData::from_raw(generate_chunk(
                    *chunk_pos,
                    cloned_seed,
                    &cloned_table,
                    // &cloned_place,
                )),
                chunk_pos,
            )
        });
        commands.spawn(GenTask(task));
    }
    gen_task.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_pos = chunk.1;
            if **save {
                chunks_to_save.push((chunk_pos, chunk.0.to_raw()));
            }
            if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
                commands.entity(chunk_entity).insert(chunk);
            }
            commands.entity(entity).despawn_recursive();
        }
    });
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunksToSave::default())
            .insert_resource(CurrentChunks::default())
            .insert_resource(ChunkQueue::default())
            .insert_resource(ToBePlaced::default())
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
            // .add_startup_system(|mut commands: Commands| {
            //     commands.insert_resource(ChunkChannel::default());
            // })
            .add_system(destroy_chunks.after(process_queue));
    }
}
