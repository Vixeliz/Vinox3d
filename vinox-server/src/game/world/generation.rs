use acap::euclid::Euclidean;
use acap::exhaustive::ExhaustiveSearch;
use acap::NearestNeighbors;
use bevy::prelude::*;

use noise::{
    BasicMulti, Blend, Fbm, HybridMulti, MultiFractal, NoiseFn, OpenSimplex, RidgedMulti,
    RotatePoint, Worley,
};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
// use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use vinox_common::{
    storage::biomes::descriptor::BiomeDescriptor,
    world::chunks::{
        positions::RelativeVoxelPos,
        storage::{
            identifier_to_name, BiomeTable, BlockData, BlockTable, ChunkData, RawChunk, CHUNK_SIZE,
        },
    },
};

#[derive(Resource, Default, Serialize, Deserialize, Deref, DerefMut, Clone)]
pub struct ToBePlaced(pub HashMap<IVec3, Vec<(UVec3, BlockData)>>);

#[derive(Resource, Default, Deref, DerefMut, Clone)]
pub struct BiomeHashmap(pub HashMap<IVec2, String>);

#[derive(Resource, Default, Deref, DerefMut, Clone)]
pub struct BiomeTree(pub ExhaustiveSearch<Euclidean<[i32; 2]>>);

pub const SEA_LEVEL: i32 = 0;

// Just some interesting stuff to look at while testing
#[allow(clippy::type_complexity)]
pub fn add_surface(
    raw_chunk: &mut ChunkData,
    pos: IVec3,
    block_types: Vec<(BlockData, i32)>,
    rng: &mut StdRng,
) {
    for z in 0..=CHUNK_SIZE - 1 {
        for y in 0..=CHUNK_SIZE - 1 {
            for x in 0..=CHUNK_SIZE - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                let relative_pos = RelativeVoxelPos(UVec3::new(x, y, z));
                if y == CHUNK_SIZE as u32 - 1 {
                    let _full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                    let _full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                    let _full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
                    if raw_chunk.get_identifier(relative_pos) != "vinox:air" {
                        // We need to add a vec for adding blocks in a new chunk when out of range
                        // raw_chunk.set(x, y, z, grass, block_table);
                    }
                } else if raw_chunk.get_identifier(RelativeVoxelPos::new(x, y + 1, z))
                    == "vinox:air"
                    && raw_chunk.get_identifier(relative_pos) != "vinox:air"
                {
                    raw_chunk.set(
                        relative_pos,
                        block_types
                            .choose_weighted(rng, |item| item.1)
                            .unwrap()
                            .clone()
                            .0,
                    );
                }
            }
        }
    }
}
#[allow(clippy::type_complexity)]
pub fn add_ceiling(
    raw_chunk: &mut ChunkData,
    pos: IVec3,
    block_types: Vec<(BlockData, i32)>,
    rng: &mut StdRng,
) {
    for z in 0..=CHUNK_SIZE - 1 {
        for y in 0..=CHUNK_SIZE - 1 {
            for x in 0..=CHUNK_SIZE - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                let relative_pos = RelativeVoxelPos(UVec3::new(x, y, z));
                if y == 0 {
                    let _full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                    let _full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                    let _full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
                    if raw_chunk.get_identifier(relative_pos) != "vinox:air" {
                        // We need to add a vec for adding blocks in a new chunk when out of range
                        // raw_chunk.set(x, y, z, grass, block_table);
                    }
                } else if raw_chunk.get_identifier(RelativeVoxelPos::new(x, y - 1, z))
                    == "vinox:air"
                    && raw_chunk.get_identifier(relative_pos) != "vinox:air"
                {
                    raw_chunk.set(
                        relative_pos,
                        block_types
                            .choose_weighted(rng, |item| item.1)
                            .unwrap()
                            .clone()
                            .0,
                    );
                }
            }
        }
    }
}

fn world_noise(seed: u32) -> impl NoiseFn<f64, 3> {
    let ridged_noise: RidgedMulti<OpenSimplex> =
        RidgedMulti::new(seed).set_octaves(4).set_frequency(0.00622);
    let d_noise: RidgedMulti<OpenSimplex> = RidgedMulti::new(seed.wrapping_add(1))
        .set_octaves(2)
        .set_frequency(0.00781);

    Blend::new(
        RotatePoint {
            source: ridged_noise,
            x_angle: 0.212,
            y_angle: 0.321,
            z_angle: -0.1204,
            u_angle: 0.11,
        },
        RotatePoint {
            source: d_noise,
            x_angle: -0.124,
            y_angle: -0.564,
            z_angle: 0.231,
            u_angle: -0.1151,
        },
        BasicMulti::<OpenSimplex>::new(seed)
            .set_octaves(1)
            .set_frequency(0.003415),
    )
}

fn values_to_biome(
    heat: i32,
    moisture: i32,
    biome_hashmap: &BiomeHashmap,
    biome_tree: &BiomeTree,
) -> String {
    if let Some(nearest) = biome_tree.0.nearest(&[heat, moisture]) {
        return biome_hashmap
            .get(&(*nearest.item.inner()).into())
            .unwrap()
            .clone();
    }
    "vinox:stone".to_string()
}

fn biome_noise(x: f64, y: f64, z: f64, seed: u32) -> (i32, i32) {
    let heat_noise = Worley::new(seed)
        .set_return_type(noise::core::worley::ReturnType::Value)
        .set_frequency(0.005022);
    let moisture_noise = Worley::new(seed.wrapping_add(1))
        .set_return_type(noise::core::worley::ReturnType::Value)
        .set_frequency(0.005022);
    (
        (heat_noise.get([x, y, z]) * 100.0) as i32,
        (moisture_noise.get([x, y, z]) * 100.0) as i32,
    )
}

// NOTE: A main design goal i have is most things should be completely generatable per chunk without needing other chunks. The only exception
// will hopefully be structures. Even then i hope to find a system where some can still be generated determinitely such as pillars.
// I like this as 1) it makes designing generation much easier and 2) makes it so you can generate any given chunk and hopefully see what itll look like
// regardless of if you are generating the neighbors
// I also just generally like procedural generation and would like to push my self to see what I can do.
pub fn generate_chunk(
    pos: IVec3,
    seed: u32,
    biome_table: &BiomeTable,
    biome_hashmap: &BiomeHashmap,
    biome_tree: &BiomeTree,
    // to_be_placed: &ToBePlaced,
) -> RawChunk {
    //TODO: Switch to using ron files to determine biomes and what blocks they should use. For now hardcoding a simplex noise
    let (heat, humidity) = biome_noise(
        pos.x as f64 * CHUNK_SIZE as f64,
        pos.y as f64 * CHUNK_SIZE as f64,
        pos.z as f64 * CHUNK_SIZE as f64,
        seed,
    );
    let main_blocks = biome_table
        .get(&values_to_biome(heat, humidity, biome_hashmap, biome_tree))
        .unwrap()
        .main_block
        .clone();
    let ridged_noise: HybridMulti<OpenSimplex> =
        HybridMulti::new(seed).set_octaves(4).set_frequency(0.02122);
    let d_noise: RidgedMulti<OpenSimplex> = RidgedMulti::new(seed.wrapping_add(1))
        .set_octaves(4)
        .set_frequency(0.01881);
    let a_noise = Fbm::<OpenSimplex>::new(seed)
        .set_octaves(3)
        .set_persistence(0.5)
        .set_frequency(0.02);

    // let final_noise = Blend::new(
    //     RotatePoint {
    //         source: ridged_noise,
    //         x_angle: 0.212,
    //         y_angle: 0.321,
    //         z_angle: -0.1204,
    //         u_angle: 0.11,
    //     },
    //     RotatePoint {
    //         source: d_noise,
    //         x_angle: -0.124,
    //         y_angle: -0.564,
    //         z_angle: 0.231,
    //         u_angle: -0.1151,
    //     },
    //     BasicMulti::<OpenSimplex>::new(seed)
    //         .set_octaves(1)
    //         .set_frequency(0.015415),
    // );
    let mut raw_chunk = ChunkData::default();
    for x in 0..=CHUNK_SIZE - 1 {
        for z in 0..=CHUNK_SIZE - 1 {
            for y in 0..=CHUNK_SIZE - 1 {
                let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y);
                let (x, y, z) = (x as u32, y as u32, z as u32);
                let relative_pos = RelativeVoxelPos(UVec3::new(x, y, z));
                let is_cave = ridged_noise
                    .get([full_x as f64, full_y as f64, full_z as f64])
                    .abs()
                    < 0.1
                    && d_noise
                        .get([full_x as f64, full_y as f64, full_z as f64])
                        .abs()
                        < 0.1
                    && (a_noise.get([full_x as f64, full_y as f64, full_z as f64]) < 0.45);
                // let noise_val =
                //     final_noise.get([full_x as f64, full_y as f64, full_z as f64]) * 45.152;
                // let noise_val =
                // world_noise(seed).get([full_x as f64, full_y as f64, full_z as f64]) * 45.152;
                if !is_cave {
                    let mut rng: StdRng = SeedableRng::seed_from_u64(
                        IVec3::new(full_x, full_y, full_z).reflect_hash().unwrap(),
                    );
                    let main_block = main_blocks
                        .choose_weighted(&mut rng, |item| item.1)
                        .unwrap()
                        .clone()
                        .0;
                    let (namespace, name) = identifier_to_name(main_block).unwrap();
                    raw_chunk.set(
                        relative_pos,
                        BlockData::new(namespace.clone(), name.clone()),
                    );
                } else {
                    raw_chunk.set(
                        relative_pos,
                        BlockData::new("vinox".to_string(), "air".to_string()),
                    );
                }
            }
        }
    }
    // add_surface(
    //     &mut raw_chunk,
    //     pos,
    //     block_table,
    //     vec![
    //         (BlockData::new("vinox".to_string(), "ignis".to_string()), 3),
    //         (BlockData::new("vinox".to_string(), "slate".to_string()), 1),
    //         (BlockData::new("vinox".to_string(), "gravel".to_string()), 1),
    //     ],
    //     &mut rng,
    // );
    // add_ceiling(
    //     &mut raw_chunk,
    //     pos,
    //     block_table,
    //     vec![
    //         (BlockData::new("vinox".to_string(), "worley".to_string()), 4),
    //         (
    //             BlockData::new("vinox".to_string(), "granite".to_string()),
    //             1,
    //         ),
    //     ],
    //     &mut rng,
    // );
    // add_blobs(
    //     &mut raw_chunk,
    //     pos,
    //     block_table,
    //     // This would be a vector of different type of biome blocks
    //     vec![
    //         (
    //             vec![(BlockData::new("vinox".to_string(), "light".to_string()), 1)],
    //             1,
    //         ),
    //         (
    //             vec![(BlockData::new("vinox".to_string(), "dirt".to_string()), 1)],
    //             1,
    //         ),
    //     ]
    //     .choose_weighted(&mut rng, |item| item.1)
    //     .unwrap()
    //     .0
    //     .clone(),
    //     seed,
    //     &mut rng,
    // );
    // add_to_be(&mut raw_chunk, pos, block_table, to_be_placed);
    raw_chunk.to_raw()
}
