use std::collections::HashMap; // for saving chunks
use bevy::prelude::*;

use crate::chunks::*;

pub const CHUNK_SIZE: usize = 64;
pub const VERTEX_SPACING: f32 = 1.0; // wie viele verticies in einem chunk sind
pub const RENDER_DISTANCE: i32 = 20;
pub const SEED: u32 = 12345;
pub const NOISE_FREQ: f64 = 0.01; // wie hart die übergänge sind
pub const NOISE_AMP: f32 = 20.0; // wie steil alles ist, also berge und so

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

#[derive(Resource, Default)]
pub struct LoadedChunks {
    pub chunks: HashMap<ChunkCoord, Entity>,
}

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LoadedChunks>()
            .add_systems(Update, chunk_system);
    }
}

pub fn chunk_system(
    mut commands: Commands,
    mut loaded: ResMut<LoadedChunks>,
    player_query: Query<&GlobalTransform, With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation();

    let cx = (player_pos.x / (CHUNK_SIZE as f32 * VERTEX_SPACING)).floor() as i32; // chunk coords 
    let cz = (player_pos.z / (CHUNK_SIZE as f32 * VERTEX_SPACING)).floor() as i32;

    let mut wanted_chunk = Vec::new();
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE { // check which chunks should be loaded
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            wanted_chunk.push(ChunkCoord { x: cx + dx, z: cz + dz });
        }
    }

    // load chunks
    for coord in wanted_chunk.iter() {
        if !loaded.chunks.contains_key(coord) {
            let mesh = calc_to_generate_chunk(*coord);

            let ent = commands.spawn((
                Mesh3d(meshes.add(mesh)), 
                MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                Transform::from_xyz(
                    coord.x as f32 * CHUNK_SIZE as f32 * VERTEX_SPACING,
                    0.0,
                    coord.z as f32 * CHUNK_SIZE as f32 * VERTEX_SPACING,
                ),
            )).id();

            loaded.chunks.insert(*coord, ent);
        }
    }

    //unload chunks
        loaded.chunks.retain(|coord, ent| {
        let dx = coord.x - cx;
        let dz = coord.z - cz;

        if dx.abs() > RENDER_DISTANCE || dz.abs() > RENDER_DISTANCE {
            commands.entity(*ent).despawn();
            false
        } else {
            true
        }
    });
}

