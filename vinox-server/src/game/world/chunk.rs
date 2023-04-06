use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use vinox_common::world::chunks::{
    ecs::{
        ChunkManager, CurrentChunks, LoadPoint, NeedsChunkData, PrepassChunk, RemoveChunk,
        SentChunks, SimulationRadius,
    },
    positions::ChunkPos,
    storage::{BiomeTable, BlockTable, ChunkData},
};

use crate::game::networking::components::SaveGame;

use super::{
    generation::generate_chunk,
    storage::{
        load_chunk, save_chunks, save_passwords, save_players, ChunksToSave, FirstSaves,
        PlayersToSave, WorldDatabase, WorldInfo,
    },
};

#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub struct PrepassEvent(pub ChunkPos);

#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub struct FullGenEvent(pub ChunkPos);

#[derive(Default, Component, Debug)]
pub struct GeneratingChunk;

pub fn generate_chunks_world(
    load_points: Query<&LoadPoint>,
    mut chunk_queue: EventWriter<PrepassEvent>,
    mut commands: Commands,
    chunk_manager: ChunkManager,
    database: Res<WorldDatabase>,
    save: Res<SaveGame>,
    no_data: Query<With<NeedsChunkData>>,
) {
    for (entity, pos) in chunk_manager
        .current_chunks
        .get_entities(load_points.iter().copied().collect::<Vec<_>>().as_slice())
    {
        if no_data.get(entity).is_ok() {
            let data = database.connection.get().unwrap();
            if let Some(chunk) = load_chunk(pos, &data) {
                if **save {
                    commands
                        .entity(entity)
                        .insert(ChunkData::from_raw(chunk))
                        .insert(pos);
                    commands.entity(entity).remove::<NeedsChunkData>();
                    continue;
                }
            }
            chunk_queue.send(PrepassEvent(pos));
            commands.entity(entity).remove::<NeedsChunkData>();
            commands.entity(entity).insert(PrepassChunk);
        }
    }
}

pub fn destroy_chunks(
    mut commands: Commands,
    remove_chunks: Query<(&ChunkPos, Entity), With<RemoveChunk>>,
    mut load_points: Query<&mut SentChunks>,
) {
    for (chunk, chunk_entity) in remove_chunks.iter() {
        for mut sent_chunks in load_points.iter_mut() {
            sent_chunks.chunks.remove(chunk);
        }
        commands.entity(chunk_entity).despawn_recursive();
    }
}

pub fn unsend_chunks(
    chunks: Query<&ChunkPos>,
    mut load_points: Query<(&LoadPoint, &mut SentChunks)>,
) {
    for (load_point, mut sent_chunks) in load_points.iter_mut() {
        for chunk in chunks.iter() {
            if !load_point.is_in_radius(chunk) {
                sent_chunks.chunks.remove(chunk);
            } else {
                continue;
            }
        }
    }
}

pub fn process_save(
    mut chunks_to_save: ResMut<ChunksToSave>,
    mut players_to_save: ResMut<PlayersToSave>,
    mut first_saves: ResMut<FirstSaves>,
    database: Res<WorldDatabase>,
) {
    save_chunks(&chunks_to_save, &database.connection.get().unwrap());
    save_passwords(&first_saves, &database.connection.get().unwrap());
    save_players(&players_to_save, &database.connection.get().unwrap());
    chunks_to_save.clear();
    players_to_save.clear();
    first_saves.clear();
}

// #[derive(Component)]
// pub struct GenTask(Task<(ChunkData, ChunkPos)>);
#[derive(Component)]
pub struct GenTask(Task<ChunkPos>);

#[derive(Component)]
pub struct PreGenTask(Task<(ChunkData, ChunkPos)>);

pub fn process_pre_queue(
    mut commands: Commands,
    mut chunk_queue: EventReader<PrepassEvent>,
    mut full_gen_queue: EventWriter<FullGenEvent>,
    mut gen_task: Query<(Entity, &mut PreGenTask)>,
    current_chunks: Res<CurrentChunks>,
    world_info: Res<WorldInfo>,
    _chunks_to_save: ResMut<ChunksToSave>,
    block_table: Res<BlockTable>,
    biome_table: Res<BiomeTable>,
    _save: Res<SaveGame>,
) {
    let cloned_seed = world_info.seed;
    let task_pool = AsyncComputeTaskPool::get();
    for event in chunk_queue.iter() {
        let chunk_pos = event.0;
        let cloned_table = biome_table.clone();
        let task = task_pool.spawn(async move {
            (
                ChunkData::from_raw(generate_chunk(*chunk_pos, cloned_seed, &cloned_table)),
                chunk_pos,
            )
        });
        commands.spawn(PreGenTask(task));
    }
    gen_task.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_pos = chunk.1;
            // if **save {
            //     chunks_to_save.push((chunk_pos, chunk.0.to_raw()));
            // }
            if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
                full_gen_queue.send(FullGenEvent(chunk_pos));
                commands
                    .entity(chunk_entity)
                    .insert(chunk)
                    .insert(GeneratingChunk);
                commands.entity(chunk_entity).remove::<PrepassChunk>();
            }
            commands.entity(entity).despawn_recursive();
        }
    });
}

pub fn process_queue(
    mut commands: Commands,
    mut chunk_queue: EventReader<FullGenEvent>,
    mut gen_task: Query<(Entity, &mut GenTask)>,
    current_chunks: Res<CurrentChunks>,
    world_info: Res<WorldInfo>,
    _chunks_to_save: ResMut<ChunksToSave>,
    _block_table: Res<BlockTable>,
    _save: Res<SaveGame>,
) {
    let _cloned_seed = world_info.seed;
    let _task_pool = AsyncComputeTaskPool::get();
    for event in chunk_queue.iter() {
        let chunk_pos = event.0;
        if let Some(chunk_entity) = current_chunks.get_entity(chunk_pos) {
            // commands.entity(chunk_entity).insert(chunk);
            commands.entity(chunk_entity).remove::<GeneratingChunk>();
        }
        // let cloned_table = block_table.clone();
        // let task = task_pool.spawn(async move { chunk_pos });
        // commands.spawn(GenTask(task));
    }
    gen_task.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            // if **save {
            //     chunks_to_save.push((chunk_pos, chunk.0.to_raw()));
            // }
            if let Some(chunk_entity) = current_chunks.get_entity(chunk) {
                // commands.entity(chunk_entity).insert(chunk);
                commands.entity(chunk_entity).remove::<GeneratingChunk>();
            }
            commands.entity(entity).despawn_recursive();
        }
    });
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ChunkSet {
    UpdateChunks,
    ReceiveChunks,
    UnloadChunks,
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunksToSave::default())
            .insert_resource(PlayersToSave::default())
            .insert_resource(FirstSaves::default())
            .insert_resource(CurrentChunks::default())
            .insert_resource(SimulationRadius {
                vertical: 4,
                horizontal: 4,
            })
            .add_systems((unsend_chunks, generate_chunks_world))
            .add_system(process_pre_queue.after(unsend_chunks))
            .add_system(process_queue.after(unsend_chunks))
            .add_system(process_save.after(process_queue))
            .add_system(destroy_chunks.after(process_queue))
            .add_event::<PrepassEvent>()
            .add_event::<FullGenEvent>();
    }
}
