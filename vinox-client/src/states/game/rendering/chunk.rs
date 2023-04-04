use bevy::sprite::TextureAtlas;
use ndshape::{ConstShape, ConstShape3usize};
use serde_big_array::Array;
use vinox_common::{
    storage::geometry::descriptor::BlockGeo,
    world::chunks::{
        positions::RelativeVoxelPos,
        storage::{trim_geo_identifier, BlockTable, ChunkData, RenderedBlockData},
    },
};

use crate::states::assets::load::LoadableAssets;

use super::meshing::GeometryTable;

const BOUNDARY_EDGE: usize = ChunkData::edge() + 2;
type BoundaryShape = ConstShape3usize<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary {
    pub geometry_pal: Vec<BlockGeo>,
    voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(
        center: ChunkData,
        neighbors: Box<[ChunkData; 26]>,
        block_table: &BlockTable,
        geo_table: &GeometryTable,
        loadable_assets: &LoadableAssets,
        texture_atlas: &TextureAtlas,
    ) -> Self {
        const MAX: usize = ChunkData::edge();
        const BOUND: usize = MAX + 1;
        let mut pal = Vec::new();
        let mut matching_voxels = Vec::new();
        let voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]> = (0..BoundaryShape::SIZE)
            .map(|idx| {
                let [x, y, z] = BoundaryShape::delinearize(idx);
                match (x, y, z) {
                    (0, 0, 0) => get_rend(
                        &neighbors[0],
                        MAX - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 0, 1..=MAX) => get_rend(
                        &neighbors[1],
                        MAX - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 0, BOUND) => get_rend(
                        &neighbors[2],
                        MAX - 1,
                        MAX - 1,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 1..=MAX, 0) => get_rend(
                        &neighbors[3],
                        MAX - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 1..=MAX, 1..=MAX) => get_rend(
                        &neighbors[4],
                        MAX - 1,
                        y - 1,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 1..=MAX, BOUND) => get_rend(
                        &neighbors[5],
                        MAX - 1,
                        y - 1,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, BOUND, 0) => get_rend(
                        &neighbors[6],
                        MAX - 1,
                        0,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, BOUND, 1..=MAX) => get_rend(
                        &neighbors[7],
                        MAX - 1,
                        0,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, BOUND, BOUND) => get_rend(
                        &neighbors[8],
                        MAX - 1,
                        0,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 0, 0) => get_rend(
                        &neighbors[9],
                        x - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 0, 1..=MAX) => get_rend(
                        &neighbors[10],
                        x - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 0, BOUND) => get_rend(
                        &neighbors[11],
                        x - 1,
                        MAX - 1,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 1..=MAX, 0) => get_rend(
                        &neighbors[12],
                        x - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 1..=MAX, 1..=MAX) => get_rend(
                        &center,
                        x - 1,
                        y - 1,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 1..=MAX, BOUND) => get_rend(
                        &neighbors[13],
                        x - 1,
                        y - 1,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, BOUND, 0) => get_rend(
                        &neighbors[14],
                        x - 1,
                        0,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, BOUND, 1..=MAX) => get_rend(
                        &neighbors[15],
                        x - 1,
                        0,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, BOUND, BOUND) => get_rend(
                        &neighbors[16],
                        x - 1,
                        0,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 0, 0) => get_rend(
                        &neighbors[17],
                        0,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 0, 1..=MAX) => get_rend(
                        &neighbors[18],
                        0,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 0, BOUND) => get_rend(
                        &neighbors[19],
                        0,
                        MAX - 1,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 1..=MAX, 0) => get_rend(
                        &neighbors[20],
                        0,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 1..=MAX, 1..=MAX) => get_rend(
                        &neighbors[21],
                        0,
                        y - 1,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 1..=MAX, BOUND) => get_rend(
                        &neighbors[22],
                        0,
                        y - 1,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, BOUND, 0) => get_rend(
                        &neighbors[23],
                        0,
                        0,
                        MAX - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, BOUND, 1..=MAX) => get_rend(
                        &neighbors[24],
                        0,
                        0,
                        z - 1,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, BOUND, BOUND) => get_rend(
                        &neighbors[25],
                        0,
                        0,
                        0,
                        geo_table,
                        block_table,
                        loadable_assets,
                        &mut pal,
                        texture_atlas,
                        &mut matching_voxels,
                    ),

                    (_, _, _) => RenderedBlockData::default(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            voxels,
            geometry_pal: pal,
        }
    }

    pub fn voxels(&self) -> &[RenderedBlockData; BoundaryShape::USIZE] {
        &self.voxels
    }

    pub const fn edge() -> usize {
        BOUNDARY_EDGE
    }

    pub const fn size() -> usize {
        BoundaryShape::SIZE
    }

    pub fn linearize(x: usize, y: usize, z: usize) -> usize {
        BoundaryShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: usize) -> (usize, usize, usize) {
        let res = BoundaryShape::delinearize(idx);
        (res[0], res[1], res[2])
    }

    pub fn x_offset() -> usize {
        ChunkBoundary::linearize(1, 0, 0) - ChunkBoundary::linearize(0, 0, 0)
    }

    pub fn y_offset() -> usize {
        ChunkBoundary::linearize(0, 1, 0) - ChunkBoundary::linearize(0, 0, 0)
    }

    pub fn z_offset() -> usize {
        ChunkBoundary::linearize(0, 0, 1) - ChunkBoundary::linearize(0, 0, 0)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_rend(
    chunk: &ChunkData,
    x: usize,
    y: usize,
    z: usize,
    geo_table: &GeometryTable,
    block_table: &BlockTable,
    loadable_assets: &LoadableAssets,
    pal: &mut Vec<BlockGeo>,
    texture_atlas: &TextureAtlas,
    matching_blocks: &mut Vec<String>,
) -> RenderedBlockData {
    let (x, y, z) = (x as u32, y as u32, z as u32);
    // return RenderedBlockData::default();
    let voxel = chunk.get(RelativeVoxelPos::new(x, y, z));
    let identifier = chunk.get_identifier(RelativeVoxelPos::new(x, y, z));
    let block_data = block_table.get(&identifier).unwrap();
    let geo_data = geo_table.get(
        &block_data
            .clone()
            .geometry
            .unwrap_or_default()
            .get_geo_namespace(),
    );
    // if block_data.clone().name.eq("water") {
    //     println!("{:?}", block_data.clone().geometry);
    // }
    let geo_data_new = geo_data.unwrap().element.clone();
    let geo_index = if pal.contains(&geo_data_new) {
        pal.iter().position(|r| r.clone() == geo_data_new).unwrap()
    } else {
        pal.push(geo_data_new.clone());
        pal.iter().position(|r| r.clone() == geo_data_new).unwrap()
    };
    let trimed_identifier = trim_geo_identifier(identifier.clone());
    let match_index = if matching_blocks.contains(&trimed_identifier) {
        matching_blocks
            .iter()
            .position(|r| r.clone().eq(&trimed_identifier))
            .unwrap()
    } else {
        matching_blocks.push(trimed_identifier.clone());
        matching_blocks
            .iter()
            .position(|r| r.clone().eq(&trimed_identifier))
            .unwrap()
    };
    let tex_variance = block_data.tex_variance.unwrap_or_default();
    let tex_variance = [
        tex_variance[0].unwrap_or(false),
        tex_variance[1].unwrap_or(false),
        tex_variance[2].unwrap_or(false),
        tex_variance[3].unwrap_or(false),
        tex_variance[4].unwrap_or(false),
        tex_variance[5].unwrap_or(false),
    ];
    let mut textures = [0, 0, 0, 0, 0, 0];
    for (i, texture) in textures.iter_mut().enumerate() {
        *texture = texture_atlas
            .get_texture_index(&loadable_assets.block_textures.get(&identifier).unwrap()[i])
            .unwrap_or_default();
    }

    RenderedBlockData {
        // identifier,
        geo_index,
        direction: voxel.direction,
        top: voxel.top,
        match_index,
        // geo: geo_data.unwrap().element.clone(),
        textures,
        visibility: block_data.visibility.unwrap_or_default(),
        has_direction: block_data.has_direction.unwrap_or(false),
        exclusive_direction: block_data.exclusive_direction.unwrap_or(false),
        tex_variance,
        blocks: geo_data.unwrap().blocks,
        light: chunk.get_light(x, y, z),
    }
}
