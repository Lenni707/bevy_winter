use bevy::prelude::*;

use noise::{Perlin, Seedable, NoiseFn}; // "random" noise generator

use bevy::render::mesh::{Mesh, Indices, PrimitiveTopology}; // for rendering specifc meshes with indices
use bevy::render::render_asset::RenderAssetUsages;

use crate::world_gen::*; // link to world gen module

// biome shit
pub const BIOME_FREQ: f64 = 0.0008;

pub const RIVER_FREQ: f64 = 0.0015;
pub const RIVER_WIDTH: f32 = 0.05;

pub const PLAINS_SCALE: f32 = 0.5;
pub const FOREST_SCALE: f32 = 1.0;
pub const MOUNTAIN_SCALE: f32 = 3.0;

pub const CLIFF_SLOPE: f32 = 0.30;
pub const SNOW_HEIGHT: f32 = 25.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    Mountains,
    River
}

pub fn calc_to_generate_chunk(coord: ChunkCoord) -> Mesh {
    let perlin = Perlin::new().set_seed(SEED);

    let mut positions = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for z in 0..CHUNK_SIZE { // calc vertices
        for x in 0..CHUNK_SIZE {
            let world_x = (coord.x * CHUNK_SIZE as i32 + x as i32) as f64;
            let world_z = (coord.z * CHUNK_SIZE as i32 + z as i32) as f64;

            let height = (perlin.get([world_x * NOISE_FREQ, world_z * NOISE_FREQ]) as f32) * NOISE_AMP; // calc height

            positions.push([x as f32 * VERTEX_SPACING, height, z as f32 * VERTEX_SPACING]);
        }
    }

    for z in 0..CHUNK_SIZE - 1 { // calc indices (triangles connecting vertices)
        for x in 0..CHUNK_SIZE - 1 {
            let i = z * CHUNK_SIZE + x;

            indices.extend_from_slice(&[
                i as u32,
                (i + CHUNK_SIZE) as u32,
                (i + 1) as u32,
                (i + 1) as u32,
                (i + CHUNK_SIZE) as u32,
                (i + CHUNK_SIZE + 1) as u32,
            ]);
        }
    }

    let mut mesh = Mesh::new( // create mesh
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions); // edit mesh based on vertices and indices
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    mesh
}