use ndshape::{ConstShape, ConstShape3usize};
use serde_big_array::Array;
use vinox_common::world::chunks::storage::{BlockTable, ChunkData, RenderedBlockData};

use super::meshing::GeometryTable;

const BOUNDARY_EDGE: usize = ChunkData::edge() + 2;
type BoundaryShape = ConstShape3usize<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary {
    voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(
        center: ChunkData,
        neighbors: Box<Array<ChunkData, 26>>,
        block_table: &BlockTable,
        geo_table: &GeometryTable,
    ) -> Self {
        const MAX: usize = ChunkData::edge();
        const BOUND: usize = MAX + 1;

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
                    ),
                    (0, 0, 1..=MAX) => get_rend(
                        &neighbors[1],
                        MAX - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                    ),
                    (0, 0, BOUND) => {
                        get_rend(&neighbors[2], MAX - 1, MAX - 1, 0, geo_table, block_table)
                    }
                    (0, 1..=MAX, 0) => get_rend(
                        &neighbors[3],
                        MAX - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (0, 1..=MAX, 1..=MAX) => {
                        get_rend(&neighbors[4], MAX - 1, y - 1, z - 1, geo_table, block_table)
                    }
                    (0, 1..=MAX, BOUND) => {
                        get_rend(&neighbors[5], MAX - 1, y - 1, 0, geo_table, block_table)
                    }
                    (0, BOUND, 0) => {
                        get_rend(&neighbors[6], MAX - 1, 0, MAX - 1, geo_table, block_table)
                    }
                    (0, BOUND, 1..=MAX) => {
                        get_rend(&neighbors[7], MAX - 1, 0, z - 1, geo_table, block_table)
                    }
                    (0, BOUND, BOUND) => {
                        get_rend(&neighbors[8], MAX - 1, 0, 0, geo_table, block_table)
                    }
                    (1..=MAX, 0, 0) => get_rend(
                        &neighbors[9],
                        x - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (1..=MAX, 0, 1..=MAX) => get_rend(
                        &neighbors[10],
                        x - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        block_table,
                    ),
                    (1..=MAX, 0, BOUND) => {
                        get_rend(&neighbors[11], x - 1, MAX - 1, 0, geo_table, block_table)
                    }
                    (1..=MAX, 1..=MAX, 0) => get_rend(
                        &neighbors[12],
                        x - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        block_table,
                    ),
                    (1..=MAX, 1..=MAX, 1..=MAX) => {
                        get_rend(&center, x - 1, y - 1, z - 1, geo_table, block_table)
                    }
                    (1..=MAX, 1..=MAX, BOUND) => {
                        get_rend(&neighbors[13], x - 1, y - 1, 0, geo_table, block_table)
                    }
                    (1..=MAX, BOUND, 0) => {
                        get_rend(&neighbors[14], x - 1, 0, MAX - 1, geo_table, block_table)
                    }
                    (1..=MAX, BOUND, 1..=MAX) => {
                        get_rend(&neighbors[15], x - 1, 0, z - 1, geo_table, block_table)
                    }
                    (1..=MAX, BOUND, BOUND) => {
                        get_rend(&neighbors[16], x - 1, 0, 0, geo_table, block_table)
                    }
                    (BOUND, 0, 0) => {
                        get_rend(&neighbors[17], 0, MAX - 1, MAX - 1, geo_table, block_table)
                    }
                    (BOUND, 0, 1..=MAX) => {
                        get_rend(&neighbors[18], 0, MAX - 1, z - 1, geo_table, block_table)
                    }
                    (BOUND, 0, BOUND) => {
                        get_rend(&neighbors[19], 0, MAX - 1, 0, geo_table, block_table)
                    }
                    (BOUND, 1..=MAX, 0) => {
                        get_rend(&neighbors[20], 0, y - 1, MAX - 1, geo_table, block_table)
                    }
                    (BOUND, 1..=MAX, 1..=MAX) => {
                        get_rend(&neighbors[21], 0, y - 1, z - 1, geo_table, block_table)
                    }
                    (BOUND, 1..=MAX, BOUND) => {
                        get_rend(&neighbors[22], 0, y - 1, 0, geo_table, block_table)
                    }
                    (BOUND, BOUND, 0) => {
                        get_rend(&neighbors[23], 0, 0, MAX - 1, geo_table, block_table)
                    }
                    (BOUND, BOUND, 1..=MAX) => {
                        get_rend(&neighbors[24], 0, 0, z - 1, geo_table, block_table)
                    }
                    (BOUND, BOUND, BOUND) => {
                        get_rend(&neighbors[25], 0, 0, 0, geo_table, block_table)
                    }

                    (_, _, _) => RenderedBlockData::default(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self { voxels }
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

pub fn get_rend(
    chunk: &ChunkData,
    x: usize,
    y: usize,
    z: usize,
    geo_table: &GeometryTable,
    block_table: &BlockTable,
) -> RenderedBlockData {
    // return RenderedBlockData::default();
    let voxel = chunk.get(x, y, z);
    let identifier = chunk.get_identifier(x, y, z);
    let block_data = block_table.get(&identifier).unwrap();
    let geo_data = geo_table.get(
        &block_data
            .clone()
            .geometry
            .unwrap_or_default()
            .get_geo_namespace(),
    );
    let tex_variance = block_data.tex_variance.unwrap_or_default();
    let tex_variance = [
        tex_variance[0].unwrap_or(false),
        tex_variance[1].unwrap_or(false),
        tex_variance[2].unwrap_or(false),
        tex_variance[3].unwrap_or(false),
        tex_variance[4].unwrap_or(false),
        tex_variance[5].unwrap_or(false),
    ];
    RenderedBlockData {
        identifier,
        direction: voxel.direction,
        top: voxel.top,
        geo: geo_data.unwrap().element.clone(),
        visibility: block_data.visibility.unwrap_or_default(),
        has_direction: block_data.has_direction.unwrap_or(false),
        exclusive_direction: block_data.exclusive_direction.unwrap_or(false),
        tex_variance,
        blocks: geo_data.unwrap().blocks,
        light: chunk.get_light(ChunkData::linearize(x, y, z)),
    }
}
