use bevy::prelude::*;
use big_space::GridCell;

use crate::world::chunks::{
    ecs::ChunkManager,
    positions::{ChunkPos, RelativeVoxelPos, VoxelPos},
};
// Takes in absolute world positions returns a chunk pos and a voxel pos for whatever face it hits and a normal
pub fn raycast_world(
    origin: Vec3,
    direction: Vec3,
    radius: f32,
    chunk_manager: &ChunkManager,
    grid_cell: &GridCell<i32>,
) -> Option<(ChunkPos, RelativeVoxelPos, Vec3, f32)> {
    // TMax needs the fractional part of origin to work.
    let mut tmax = Vec3::new(
        intbound(origin.x, direction.x),
        intbound(origin.y, direction.y),
        intbound(origin.z, direction.z),
    );

    let mut current_block = VoxelPos::from(origin).as_vec3();
    let step = direction.signum();

    let tdelta = step / direction;

    let mut face = Vec3::ZERO;

    if direction == Vec3::ZERO {
        return None;
    }

    let radius = radius
        / (direction.x * direction.x + direction.y * direction.y + direction.z * direction.z)
            .sqrt();
    let mut lastmax = 0.0;
    let mut counter = 0;
    loop {
        // Infinite loop shouldve been prevented by tmax but it isn't for some reason all the time. This just breaks the loop after 4 x the radius which should be plenty of time to find the voxel
        // It could be due to chunks or neighbors not existing?
        if counter > (radius * 4.0) as u32 {
            break;
        }
        let final_translation = Vec3::new(
            (grid_cell.x * 10000) as f32 + current_block.x,
            (grid_cell.y * 10000) as f32 + current_block.y,
            (grid_cell.z * 10000) as f32 + current_block.z,
        );
        let (voxel_pos, chunk_pos) = VoxelPos::from(final_translation).to_offsets();
        if let Some(block) = chunk_manager.get_block(VoxelPos::from(final_translation)) {
            if !block.is_empty(&chunk_manager.block_table) {
                let toi = lastmax * direction.length();
                return Some((chunk_pos, voxel_pos, face, toi));
            }
        }

        if tmax.x < tmax.y {
            if tmax.x < tmax.z {
                if tmax.x > radius {
                    break;
                }
                lastmax = tmax.x;
                current_block.x += step.x;
                tmax.x += tdelta.x;
                face.x = -step.x;
                face.y = 0.0;
                face.z = 0.0;
            } else {
                if tmax.z > radius {
                    break;
                }
                lastmax = tmax.z;
                current_block.z += step.z;
                tmax.z += tdelta.z;
                face.x = 0.0;
                face.y = 0.0;
                face.z = -step.z;
            }
        } else if tmax.y < tmax.z {
            if tmax.y > radius {
                break;
            }
            lastmax = tmax.y;
            current_block.y += step.y;
            tmax.y += tdelta.y;
            face.x = 0.0;
            face.y = -step.y;
            face.z = 0.0;
        } else {
            if tmax.z > radius {
                break;
            }
            lastmax = tmax.z;
            current_block.z += step.z;
            tmax.z += tdelta.z;
            face.x = 0.0;
            face.y = 0.0;
            face.z = -step.z;
        }
        counter += 1;
    }
    None
}

fn intbound(s: f32, ds: f32) -> f32 {
    if ds < 0.0 {
        intbound(-s, -ds)
    } else {
        let s = s.rem_euclid(1.0);
        (1.0 - s) / ds
    }
}
