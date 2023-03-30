use bevy::prelude::*;
use bracket_noise::prelude::*;
use noise::{
    BasicMulti, Billow, Blend, Cache, Clamp, Curve, Fbm, HybridMulti, Min, MultiFractal, NoiseFn,
    OpenSimplex, Perlin, RidgedMulti, RotatePoint, ScaleBias, SuperSimplex, Worley,
};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;
// use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use vinox_common::{
    storage::blocks::descriptor::BlockDescriptor,
    world::chunks::storage::{BlockData, BlockTable, ChunkData, RawChunk, CHUNK_SIZE},
};

#[derive(Resource, Default, Serialize, Deserialize, Deref, DerefMut, Clone)]
pub struct ToBePlaced(pub HashMap<IVec3, Vec<(UVec3, BlockData)>>);

pub const SEA_LEVEL: i32 = 0;

// Just some interesting stuff to look at while testing
#[allow(clippy::type_complexity)]
pub fn add_surface(
    raw_chunk: &mut ChunkData,
    pos: IVec3,
    block_table: &BlockTable,
    block_types: Vec<(BlockData, i32)>,
    rng: &mut StdRng,
) {
    for z in 0..=CHUNK_SIZE - 1 {
        for y in 0..=CHUNK_SIZE - 1 {
            for x in 0..=CHUNK_SIZE - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                if y == CHUNK_SIZE as u32 - 1 {
                    let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                    let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                    let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
                    if raw_chunk.get_identifier(x, y, z) != "vinox:air" {
                        // We need to add a vec for adding blocks in a new chunk when out of range
                        // raw_chunk.set(x, y, z, grass, block_table);
                    }
                } else if raw_chunk.get_identifier(x, y + 1, z) == "vinox:air"
                    && raw_chunk.get_identifier(x, y, z) != "vinox:air"
                {
                    raw_chunk.set(
                        x,
                        y,
                        z,
                        block_types
                            .choose_weighted(rng, |item| item.1)
                            .unwrap()
                            .clone()
                            .0,
                        block_table,
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
    block_table: &BlockTable,
    block_types: Vec<(BlockData, i32)>,
    rng: &mut StdRng,
) {
    for z in 0..=CHUNK_SIZE - 1 {
        for y in 0..=CHUNK_SIZE - 1 {
            for x in 0..=CHUNK_SIZE - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                if y == 0 {
                    let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                    let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                    let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
                    if raw_chunk.get_identifier(x, y, z) != "vinox:air" {
                        // We need to add a vec for adding blocks in a new chunk when out of range
                        // raw_chunk.set(x, y, z, grass, block_table);
                    }
                } else if raw_chunk.get_identifier(x, y - 1, z) == "vinox:air"
                    && raw_chunk.get_identifier(x, y, z) != "vinox:air"
                {
                    raw_chunk.set(
                        x,
                        y,
                        z,
                        block_types
                            .choose_weighted(rng, |item| item.1)
                            .unwrap()
                            .clone()
                            .0,
                        block_table,
                    );
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn add_blobs(
    raw_chunk: &mut ChunkData,
    pos: IVec3,
    block_table: &BlockTable,
    block_types: Vec<(BlockData, i32)>,
    seed: u32,
    rng: &mut StdRng,
) {
    let blob_noise = BasicMulti::<OpenSimplex>::new(seed)
        .set_octaves(1)
        .set_frequency(0.1531515);
    for z in 0..=CHUNK_SIZE - 1 {
        for y in 0..=CHUNK_SIZE - 1 {
            for x in 0..=CHUNK_SIZE - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
                if raw_chunk.get(x, y, z).is_opaque(block_table) {
                    let final_noise = blob_noise.get([full_x as f64, full_y as f64, full_z as f64]);
                    if final_noise < -0.15 {
                        raw_chunk.set(
                            x,
                            y,
                            z,
                            block_types
                                .choose_weighted(rng, |item| item.1)
                                .unwrap()
                                .clone()
                                .0,
                            block_table,
                        );
                    }
                }
            }
        }
    }
}

pub fn add_to_be(
    raw_chunk: &mut ChunkData,
    pos: IVec3,
    block_table: &BlockTable,
    to_be_placed: &ToBePlaced,
) {
    if let Some(blocks) = to_be_placed.get(&pos) {
        for block in blocks.iter() {
            raw_chunk.set(
                block.0.x,
                block.0.y,
                block.0.z,
                block.1.clone(),
                block_table,
            );
        }
    }
}

pub fn add_sea(raw_chunk: &mut ChunkData, pos: IVec3, block_table: &BlockTable) {
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y);
                let (x, y, z) = (x as u32, y as u32, z as u32);
                if full_y == SEA_LEVEL && raw_chunk.get(x, y, z).is_empty(block_table) {
                    let water = BlockData::new("vinox".to_string(), "water.divot".to_string());
                    raw_chunk.set(x, y, z, water, block_table);
                } else if full_y < SEA_LEVEL && raw_chunk.get(x, y, z).is_empty(block_table) {
                    let water = BlockData::new("vinox".to_string(), "water".to_string());
                    raw_chunk.set(x, y, z, water, block_table);
                }
            }
        }
    }
}

// TODO: Was going to add trees like this but instead we will do a more flexible structure system with ron
// pub fn add_trees(
//     raw_chunk: &mut RawChunk,
//     noisefn: &noise::Blend<
//         f64,
//         noise::RotatePoint<noise::RidgedMulti<noise::OpenSimplex>>,
//         noise::RotatePoint<noise::RidgedMulti<noise::OpenSimplex>>,
//         noise::BasicMulti<noise::OpenSimplex>,
//         3,
//     >,
//     pos: IVec3,
//     seed: u32,
// ) {
//     let mut rng: StdRng = SeedableRng::seed_from_u64(seed as u64);
//     for i in 0..=rng.gen_range(0..=3) {
//         for y in 0..=CHUNK_SIZE - 1 {
//             let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
//             let (tree_x, tree_z) = (
//                 rng.gen_range(0..=CHUNK_SIZE_ARR),
//                 rng.gen_range(0..=CHUNK_SIZE_ARR),
//             );
//             if y == 0 {
//                 let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) - 1;
//                 let full_x = tree_x as i32 + ((CHUNK_SIZE as i32) * pos.x);
//                 let full_z = tree_z as i32 + ((CHUNK_SIZE as i32) * pos.z);
//                 let noise_val = noisefn.get([full_x as f64, full_y as f64, full_z as f64]) * 45.152;
//                 if full_y as f64 <= noise_val
//                     && raw_chunk.get_identifier(UVec3::new(tree_x, y, tree_z)) == "vinox:air"
//                 {
//                     let wood = BlockData::new("vinox".to_string(), "cobblestone".to_string());
//                     raw_chunk.set_block(UVec3::new(tree_x, y, tree_z), &wood);
//                 }
//             } else if raw_chunk.get_identifier(UVec3::new(tree_x, y - 1, tree_z)) != "vinox:air"
//                 && raw_chunk.get_identifier(UVec3::new(tree_x, y, tree_z)) == "vinox:air"
//             {
//                 let wood = BlockData::new("vinox".to_string(), "cobblestone".to_string());
//                 raw_chunk.set_block(UVec3::new(tree_x, y, tree_z), &wood);
//             }
//         }
//     }
// }

// pub fn add_missing_blocks(raw_chunk: &mut RawChunk, to_be_placed: &ToBePlaced) {}

fn world_noise(seed: u32) -> impl NoiseFn<f64, 3> {
    let ridged_noise: RidgedMulti<OpenSimplex> =
        RidgedMulti::new(seed).set_octaves(4).set_frequency(0.00622);
    let d_noise: RidgedMulti<OpenSimplex> = RidgedMulti::new(seed.wrapping_add(1))
        .set_octaves(2)
        .set_frequency(0.00781);
    let final_noise = Blend::new(
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
    );
    final_noise
}

// NOTE: A main design goal i have is most things should be completely generatable per chunk without needing other chunks. The only exception
// will hopefully be structures. Even then i hope to find a system where some can still be generated determinitely such as pillars.
// I like this as 1) it makes designing generation much easier and 2) makes it so you can generate any given chunk and hopefully see what itll look like
// regardless of if you are generating the neighbors
// I also just generally like procedural generation and would like to push my self to see what I can do.
pub fn generate_chunk(
    pos: IVec3,
    seed: u32,
    block_table: &BlockTable,
    // to_be_placed: &ToBePlaced,
) -> RawChunk {
    //TODO: Switch to using ron files to determine biomes and what blocks they should use. For now hardcoding a simplex noise
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed as u64);
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
                    raw_chunk.set(
                        x,
                        y,
                        z,
                        BlockData::new("vinox".to_string(), "stone".to_string()),
                        block_table,
                    );
                } else {
                    raw_chunk.set(
                        x,
                        y,
                        z,
                        BlockData::new("vinox".to_string(), "air".to_string()),
                        block_table,
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
