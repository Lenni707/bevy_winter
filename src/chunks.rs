use bevy::prelude::*;

use noise::NoiseFn; // neede for .get func on noise

use bevy::render::mesh::{Mesh, Indices, PrimitiveTopology}; // for rendering specifc meshes with indices
use bevy::render::render_asset::RenderAssetUsages;

use crate::world_gen::*; // link to world gen module
use crate::noise::NoiseGenerators;

// biome shit
pub const BIOME_FREQ: f64 = 0.008;

pub const PLAINS_SCALE: f32 = 2.0;
pub const FOREST_SCALE: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
}

pub fn calc_to_generate_chunk(coord: ChunkCoord, noise: &NoiseGenerators,) -> Mesh {
    let mut positions = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let stride = CHUNK_SIZE + 1;

    for z in 0..=CHUNK_SIZE { // calc vertices
        for x in 0..=CHUNK_SIZE {
            let world_x = ((coord.x * CHUNK_SIZE as i32 + x as i32) as f64) * VERTEX_SPACING as f64;
            let world_z = ((coord.z * CHUNK_SIZE as i32 + z as i32) as f64) * VERTEX_SPACING as f64;

            let height = get_height(world_x, world_z, noise);

            positions.push([
                x as f32 * VERTEX_SPACING,
                height,
                z as f32 * VERTEX_SPACING,
            ]);

            // // height spcifc colour of snow
            // let h_norm = (height / 30.0).clamp(0.0, 1.0);

            // let base_snow = Vec3::new(0.85, 0.9, 1.0);
            // let tinted_snow = Vec3::new(0.75, 0.8, 0.95);

            // let c = base_snow * (1.0 - h_norm) + tinted_snow * h_norm;

            // colors.push([c.x, c.y, c.z, 1.0]);

            // // biom specifc colour
            // let color = match biome {
            //     Biome::Plains   => [1.0, 1.0, 1.0, 1.0],
            //     Biome::Forest   => [1.0, 1.0, 1.0, 1.0],
            // };

            // colors.push(color);
            
            // einfach so ein bisschen variation
            let n = noise.height.get([world_x * 0.15, world_z * 0.15]) as f32;

            let variation = n * 0.03;

            let r = (0.88 + variation * 0.4).clamp(0.0, 1.0);
            let g = (0.93 + variation * 0.6).clamp(0.0, 1.0);
            let b = (1.00 + variation * 1.0).clamp(0.0, 1.0);

            colors.push([r, g, b, 1.0]);
        }
    }

    for z in 0..CHUNK_SIZE { // calc indices (triangles connecting vertices)
        for x in 0..CHUNK_SIZE {
            let i = z * stride + x;

            indices.extend_from_slice(&[
                i as u32,
                (i + stride) as u32,
                (i + 1) as u32,
                (i + 1) as u32,
                (i + stride) as u32,
                (i + stride + 1) as u32,
            ]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    mesh
}


pub fn get_height(world_x: f64, world_z: f64, noise: &NoiseGenerators) -> f32 {
    // calc für biomes
        let biome_val = noise.biome.get([world_x * BIOME_FREQ, world_z * BIOME_FREQ]) as f32;

        // // check which biome it is based on noise level /brauch man nur fuer farbe ändern je nach biom
        // let biome = if biome_val < -0.2 {
        //     Biome::Plains
        // } else {
        //     Biome::Forest
        // };

        // biome blend factor t (0 = plains, 1 = forest)
        let t = ((biome_val + 0.2) / (0.3 + 0.2)).clamp(0.0, 1.0);

        // base height from noise
        let base_h = noise.height.get([world_x * NOISE_FREQ, world_z * NOISE_FREQ]) as f32;

        // height per biome
        let plains_h = base_h * NOISE_AMP * PLAINS_SCALE;
        let forest_h = base_h * NOISE_AMP * FOREST_SCALE;

        // final smooth height
        let mut height = plains_h * (1.0 - t) + forest_h * t;

        let erosion = noise.height.get([world_x * 0.03, world_z * 0.03]) as f32 * 2.0;

        // rounded snow drifts
        let drift_n = noise.height.get([world_x * 0.01, world_z * 0.01]) as f32;
        let drifts = drift_n.abs().powf(2.0) * 6.0;

        height += erosion + drifts;
        // returns height
        height
}


pub fn should_tree_spawn(
    world_x: f64,
    world_z: f64,
    noise: &NoiseGenerators,
) -> bool {
    let biome_val = noise.biome.get([world_x * BIOME_FREQ, world_z * BIOME_FREQ]) as f32;
   
    let t = ((biome_val + 0.2) / 0.5).clamp(0.0, 1.0);

    let tree_frequency = lerp(0.03, 0.25, t);

    let tree_noise = noise.tree.get([world_x * 0.14, world_z * 0.14]) as f32; // how close they spawn together somewhere here
    let tree_noise = (tree_noise + 1.0) * 0.5;

    tree_noise < tree_frequency
}

// linear interpolation helper von chatty
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}