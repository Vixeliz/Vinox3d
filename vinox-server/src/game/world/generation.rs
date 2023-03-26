use bevy::prelude::*;
use bracket_noise::prelude::*;
use std::collections::HashMap;
// use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use vinox_common::{
    storage::blocks::descriptor::BlockDescriptor,
    world::chunks::storage::{BlockData, BlockTable, ChunkData, RawChunk, CHUNK_SIZE},
};

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct ToBePlaced(HashMap<IVec3, Vec<(UVec3, BlockDescriptor)>>);

// Just some interesting stuff to look at while testing
#[allow(clippy::type_complexity)]
pub fn add_grass(
    raw_chunk: &mut ChunkData,
    // noisefn: &noise::Blend<
    //     f64,
    //     noise::RotatePoint<noise::RidgedMulti<noise::OpenSimplex>>,
    //     noise::RotatePoint<noise::RidgedMulti<noise::OpenSimplex>>,
    //     noise::BasicMulti<noise::OpenSimplex>,
    //     3,
    // >,
    noisefn: &FastNoise,
    pos: IVec3,
    block_table: &BlockTable,
) {
    for z in 0..=CHUNK_SIZE - 1 {
        for y in 0..=CHUNK_SIZE - 1 {
            for x in 0..=CHUNK_SIZE - 1 {
                let (x, y, z) = (x as u32, y as u32, z as u32);
                if y == CHUNK_SIZE as u32 - 1 {
                    let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                    let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                    let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y) + 1;
                    let noise_val =
                        noisefn.get_noise3d(full_x as f32, full_y as f32, full_z as f32) * 120.412;
                    if full_y as f32 > noise_val && raw_chunk.get_identifier(x, y, z) != "vinox:air"
                    {
                        let grass = BlockData::new("vinox".to_string(), "grass".to_string());
                        raw_chunk.set(x, y, z, grass, block_table);
                    }
                } else if raw_chunk.get_identifier(x, y + 1, z) == "vinox:air"
                    && raw_chunk.get_identifier(x, y, z) != "vinox:air"
                {
                    let grass = BlockData::new("vinox".to_string(), "grass".to_string());
                    raw_chunk.set(x, y, z, grass, block_table);
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

pub fn generate_chunk(pos: IVec3, seed: u32, block_table: &BlockTable) -> RawChunk {
    //TODO: Switch to using ron files to determine biomes and what blocks they should use. For now hardcoding a simplex noise
    // let ridged_noise: RidgedMulti<OpenSimplex> =
    //     RidgedMulti::new(seed).set_octaves(4).set_frequency(0.00622);
    // let d_noise: RidgedMulti<OpenSimplex> = RidgedMulti::new(seed.wrapping_add(1))
    //     .set_octaves(2)
    //     .set_frequency(0.00781);
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
    //         .set_frequency(0.003415),
    // );
    let mut noise = FastNoise::seeded(seed as u64);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::RigidMulti);
    noise.set_fractal_octaves(8);
    noise.set_fractal_gain(0.55);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(0.0005);

    let mut raw_chunk = ChunkData::default();
    for x in 0..=CHUNK_SIZE - 1 {
        for z in 0..=CHUNK_SIZE - 1 {
            for y in 0..=CHUNK_SIZE - 1 {
                let full_x = x as i32 + ((CHUNK_SIZE as i32) * pos.x);
                let full_z = z as i32 + ((CHUNK_SIZE as i32) * pos.z);
                let full_y = y as i32 + ((CHUNK_SIZE as i32) * pos.y);
                let (x, y, z) = (x as u32, y as u32, z as u32);
                // let noise_val =
                //     final_noise.get([full_x as f64, full_y as f64, full_z as f64]) * 45.152;
                let noise_val =
                    noise.get_noise3d(full_x as f32, full_y as f32, full_z as f32) * 120.412;
                if full_y as f32 <= noise_val {
                    raw_chunk.set(
                        x,
                        y,
                        z,
                        BlockData::new("vinox".to_string(), "dirt".to_string()),
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
    add_grass(&mut raw_chunk, &noise, pos, block_table);
    raw_chunk.to_raw()
}
