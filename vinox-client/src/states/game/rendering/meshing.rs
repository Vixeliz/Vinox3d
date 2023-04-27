use bevy::{
    math::Vec3A,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::Indices,
        primitives::Aabb,
    },
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, Task},
    utils::FloatOrd,
};
use vinox_mesher::chunk_boundary::ChunkBoundary;
use vinox_mesher::mesh::full_mesh;
use vinox_mesher::mesh::GeometryTable;
use vinox_mesher::mesh::MeshedChunk;

use big_space::FloatingOriginSettings;
use futures_lite::future;
use itertools::Itertools;


// use rand::seq::IteratorRandom;



use vinox_common::{
    world::chunks::{
        ecs::{
            ChunkManager, CurrentChunks, LoadableAssets, NeedsChunkData, NeedsMesh, PriorityMesh,
        },
        positions::{ChunkPos, RelativeVoxelPos, VoxelPos},
        storage::{
            BlockTable, ChunkData, CHUNK_SIZE,
        },
    },
};

use crate::states::{
    components::GameOptions,
    game::world::chunks::{PlayerBlock, PlayerChunk},
};

#[derive(Bundle)]
pub struct RenderedChunk {
    #[bundle]
    pub mesh: PbrBundle,
    pub aabb: Aabb,
}

#[derive(Component)]
pub struct ComputeMesh(Task<MeshedChunk>);

#[derive(Component)]
pub struct PriorityComputeMesh(Task<MeshedChunk>);

pub fn process_priority_task(
    mut commands: Commands,
    mut mesh_tasks: Query<(Entity, &mut PriorityComputeMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
    current_chunks: Res<CurrentChunks>,
    floating_settings: Res<FloatingOriginSettings>,
) {
    mesh_tasks.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(chunk_entity) = current_chunks.get_entity(chunk.pos) {
                commands.entity(chunk_entity).despawn_descendants();

                let (grid_cell, chunk_pos) =
                    floating_settings.imprecise_translation_to_grid::<i32>(chunk.pos.into());

                let trans_entity = commands
                    .spawn((
                        RenderedChunk {
                            aabb: Aabb {
                                center: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                                half_extents: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                            },
                            mesh: MaterialMeshBundle {
                                mesh: meshes.add(chunk.transparent_mesh.clone()),
                                material: chunk_material.transparent.clone(),
                                ..Default::default()
                            },
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();

                commands
                    .entity(chunk_entity)
                    .insert((
                        RenderedChunk {
                            aabb: Aabb {
                                center: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                                half_extents: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                            },
                            mesh: MaterialMeshBundle {
                                mesh: meshes.add(chunk.chunk_mesh),
                                material: chunk_material.opaque.clone(),
                                transform: Transform::from_translation(chunk_pos),
                                ..Default::default()
                            },
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .insert(grid_cell);

                commands.entity(chunk_entity).push_children(&[trans_entity]);
                commands.entity(entity).despawn_recursive();
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    });
}

pub fn process_task(
    mut commands: Commands,
    mut mesh_tasks: Query<(Entity, &mut ComputeMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
    _chunks: Query<&ChunkPos, With<NeedsMesh>>,
    _player_chunk: Res<PlayerChunk>,
    current_chunks: Res<CurrentChunks>,
) {
    mesh_tasks.for_each_mut(|(entity, mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(chunk_entity) = current_chunks.get_entity(chunk.pos) {
                commands.entity(chunk_entity).despawn_descendants();

                let chunk_pos = Vec3::new(
                    (chunk.pos.x * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.y * (CHUNK_SIZE) as i32) as f32,
                    (chunk.pos.z * (CHUNK_SIZE) as i32) as f32,
                );

                let trans_entity = commands
                    .spawn((
                        RenderedChunk {
                            aabb: Aabb {
                                center: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                                half_extents: Vec3A::new(
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                    (CHUNK_SIZE / 2) as f32,
                                ),
                            },
                            mesh: MaterialMeshBundle {
                                mesh: meshes.add(chunk.transparent_mesh.clone()),
                                material: chunk_material.transparent.clone(),
                                ..Default::default()
                            },
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();

                commands.entity(chunk_entity).insert((
                    RenderedChunk {
                        aabb: Aabb {
                            center: Vec3A::new(
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                            ),
                            half_extents: Vec3A::new(
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                                (CHUNK_SIZE / 2) as f32,
                            ),
                        },
                        mesh: MaterialMeshBundle {
                            mesh: meshes.add(chunk.chunk_mesh),
                            material: chunk_material.opaque.clone(),
                            transform: Transform::from_translation(chunk_pos),
                            ..Default::default()
                        },
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));

                commands.entity(chunk_entity).push_children(&[trans_entity]);
                commands.entity(entity).despawn_recursive();
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    });
}

pub fn priority_mesh(
    mut commands: Commands,
    chunks: Query<&ChunkPos, With<PriorityMesh>>,
    chunk_manager: ChunkManager,
    loadable_assets: ResMut<LoadableAssets>,
    block_table: Res<BlockTable>,
    geo_table: Res<GeometryTable>,
    texture_atlas: Res<Assets<TextureAtlas>>,
) {
    for chunk in chunks.iter() {
        if let Some(neighbors) = chunk_manager.get_neighbors(*chunk) {
            if let Ok(neighbors) = neighbors.try_into() {
                if let Some(chunk_entity) = chunk_manager.current_chunks.get_entity(*chunk) {
                    if let Some(chunk_data) = chunk_manager.get_chunk(chunk_entity) {
                        let chunk = *chunk;
                        let task_pool = ComputeTaskPool::get();
                        let block_atlas: TextureAtlas = texture_atlas
                            .get(&loadable_assets.block_atlas)
                            .unwrap()
                            .clone();
                        let cloned_table: BlockTable = block_table.clone();
                        let cloned_geo_table: GeometryTable = geo_table.clone();
                        let cloned_assets: LoadableAssets = loadable_assets.clone();
                        let clone_atlas: TextureAtlas = block_atlas.clone();

                        let task = task_pool.spawn(async move {
                            let raw_chunk = ChunkBoundary::new(
                                chunk_data,
                                neighbors,
                                &cloned_table,
                                &cloned_geo_table,
                                &cloned_assets,
                                &clone_atlas,
                            );
                            full_mesh(&raw_chunk, &clone_atlas, *chunk)
                        });
                        commands.spawn(PriorityComputeMesh(task));

                        commands.entity(chunk_entity).remove::<PriorityMesh>();
                        commands.entity(chunk_entity).remove::<NeedsMesh>();
                    }
                }
            }
        }
    }
}

pub fn build_mesh(
    mut commands: Commands,
    chunk_manager: ChunkManager,
    chunks: Query<&ChunkPos, (With<NeedsMesh>, Without<NeedsChunkData>)>,
    player_chunk: Res<PlayerChunk>,
    options: Res<GameOptions>,
    loadable_assets: ResMut<LoadableAssets>,
    block_table: Res<BlockTable>,
    geo_table: Res<GeometryTable>,
    texture_atlas: Res<Assets<TextureAtlas>>,
) {
    for (count, chunk) in chunks
        .iter()
        .sorted_unstable_by_key(|key| {
            FloatOrd(key.as_vec3().distance(player_chunk.chunk_pos.as_vec3()))
        })
        .enumerate()
    {
        if count > options.meshes_frame {
            return;
        }
        if let Some(neighbors) = chunk_manager.get_neighbors(*chunk) {
            if let Ok(neighbors) = neighbors.try_into() {
                if let Some(chunk_entity) = chunk_manager.current_chunks.get_entity(*chunk) {
                    if let Some(chunk_data) = chunk_manager.get_chunk(chunk_entity) {
                        let chunk = *chunk;
                        let task_pool = AsyncComputeTaskPool::get();
                        let block_atlas: TextureAtlas = texture_atlas
                            .get(&loadable_assets.block_atlas)
                            .unwrap()
                            .clone();
                        let cloned_table: BlockTable = block_table.clone();
                        let cloned_geo_table: GeometryTable = geo_table.clone();
                        let cloned_assets: LoadableAssets = loadable_assets.clone();
                        let clone_atlas: TextureAtlas = block_atlas.clone();

                        let task = task_pool.spawn(async move {
                            let raw_chunk = ChunkBoundary::new(
                                chunk_data,
                                neighbors,
                                &cloned_table,
                                &cloned_geo_table,
                                &cloned_assets,
                                &clone_atlas,
                            );
                            full_mesh(&raw_chunk, &clone_atlas, *chunk)
                        });
                        commands.spawn(ComputeMesh(task));
                    }
                    commands.entity(chunk_entity).remove::<NeedsMesh>();
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ChunkMaterial {
    opaque: Handle<StandardMaterial>,
    transparent: Handle<StandardMaterial>,
}

pub fn create_chunk_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_material: ResMut<ChunkMaterial>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    loadable_assets: ResMut<LoadableAssets>,
) {
    chunk_material.transparent = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            texture_atlas
                .get(&loadable_assets.block_atlas)
                .unwrap()
                .texture
                .clone(),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });
    chunk_material.opaque = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            texture_atlas
                .get(&loadable_assets.block_atlas)
                .unwrap()
                .texture
                .clone(),
        ),
        alpha_mode: AlphaMode::Mask(0.5),
        perceptual_roughness: 1.0,
        ..Default::default()
    });
}

pub fn priority_player(
    player_chunk: Res<PlayerChunk>,
    current_chunks: Res<CurrentChunks>,
    chunks: Query<&Handle<Mesh>>,
    mut commands: Commands,
) {
    if let Some(chunk_entity) = current_chunks.get_entity(player_chunk.chunk_pos) {
        if chunks.get(chunk_entity).is_err() {
            commands.entity(chunk_entity).insert(PriorityMesh);
        }
    }
}

pub struct SortFaces {
    chunk_pos: ChunkPos,
}

pub fn sort_faces(
    current_chunks: Res<CurrentChunks>,
    handles: Query<&Handle<Mesh>>,
    chunks: Query<&Children, With<ChunkData>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_transform: Query<&GlobalTransform, With<Camera>>,
    mut events: EventReader<SortFaces>,
) {
    for evt in events.iter() {
        if let Ok(camera_transform) = camera_transform.get_single() {
            if let Some(chunk_entity) = current_chunks.get_entity(evt.chunk_pos) {
                if let Ok(children) = chunks.get(chunk_entity) {
                    if let Some(child_entity) = children.get(0) {
                        if let Ok(chunk_mesh_handle) = handles.get(*child_entity) {
                            if let Some(chunk_mesh) = meshes.get_mut(chunk_mesh_handle) {
                                let mut collected_indices = Vec::new();
                                let mut sorted_indices: Vec<([usize; 6], f32)> = Vec::new();
                                if let Some(vertex_array) =
                                    chunk_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                                {
                                    if let Some(raw_array) = vertex_array.as_float3() {
                                        if let Some(indices) = chunk_mesh.indices() {
                                            for indice in indices.iter().chunks(6).into_iter() {
                                                let vec_ind: Vec<usize> = indice.collect();
                                                let x = (raw_array[vec_ind[1]][0]
                                                    + raw_array[vec_ind[3]][0]
                                                    + raw_array[vec_ind[4]][0]
                                                    + raw_array[vec_ind[5]][0])
                                                    / 4.0;
                                                let y = (raw_array[vec_ind[1]][1]
                                                    + raw_array[vec_ind[3]][1]
                                                    + raw_array[vec_ind[4]][1]
                                                    + raw_array[vec_ind[5]][1])
                                                    / 4.0;
                                                let z = (raw_array[vec_ind[1]][2]
                                                    + raw_array[vec_ind[3]][2]
                                                    + raw_array[vec_ind[4]][2]
                                                    + raw_array[vec_ind[5]][2])
                                                    / 4.0;
                                                let real_pos = VoxelPos::from((
                                                    RelativeVoxelPos(UVec3::new(
                                                        x as u32, y as u32, z as u32,
                                                    )),
                                                    evt.chunk_pos,
                                                ));
                                                let dist = camera_transform
                                                    .translation()
                                                    .distance(real_pos.as_vec3());
                                                sorted_indices.push((
                                                    [
                                                        vec_ind[0], vec_ind[1], vec_ind[2],
                                                        vec_ind[3], vec_ind[4], vec_ind[5],
                                                    ],
                                                    dist,
                                                ));
                                            }
                                            sorted_indices
                                                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                                            sorted_indices.reverse();

                                            // This is horrible most definitely a better way to do this
                                            for indice in sorted_indices.iter() {
                                                collected_indices.push(indice.0[0] as u32);
                                                collected_indices.push(indice.0[1] as u32);
                                                collected_indices.push(indice.0[2] as u32);
                                                collected_indices.push(indice.0[3] as u32);
                                                collected_indices.push(indice.0[4] as u32);
                                                collected_indices.push(indice.0[5] as u32);
                                            }
                                        }
                                    }
                                }

                                chunk_mesh.set_indices(Some(Indices::U32(collected_indices)));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn sort_chunks(
    player_chunk: Res<PlayerChunk>,
    player_block: Res<PlayerBlock>,
    mut sort_face: EventWriter<SortFaces>,
) {
    if player_chunk.is_changed() {
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos,
        });
        sort_face.send(SortFaces {
            chunk_pos: ChunkPos(*player_chunk.chunk_pos + IVec3::new(1, 0, 0)),
        });
        sort_face.send(SortFaces {
            chunk_pos: ChunkPos(*player_chunk.chunk_pos + IVec3::new(-1, 0, 0)),
        });
        sort_face.send(SortFaces {
            chunk_pos: ChunkPos(*player_chunk.chunk_pos + IVec3::new(0, 1, 0)),
        });
        sort_face.send(SortFaces {
            chunk_pos: ChunkPos(*player_chunk.chunk_pos + IVec3::new(0, -1, 0)),
        });
        sort_face.send(SortFaces {
            chunk_pos: ChunkPos(*player_chunk.chunk_pos + IVec3::new(0, 0, 1)),
        });
        sort_face.send(SortFaces {
            chunk_pos: ChunkPos(*player_chunk.chunk_pos + IVec3::new(0, 0, -1)),
        });
    }

    if player_block.is_changed() {
        sort_face.send(SortFaces {
            chunk_pos: player_chunk.chunk_pos,
        });
    }
}
