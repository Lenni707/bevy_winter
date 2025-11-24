use std::collections::HashMap; // for saving chunks
use bevy::prelude::*;

use crate::chunks::*;
use crate::noise::NoiseGenerators;

pub const CHUNK_SIZE: usize = 32;
pub const VERTEX_SPACING: f32 = 3.0; // wie viele verticies in einem chunk sind
pub const RENDER_DISTANCE: i32 = 10;
pub const NOISE_FREQ: f64 = 0.01; // wie hart die übergänge sind
pub const NOISE_AMP: f32 = 10.0; // wie steil alles ist, also berge und so

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
            .add_systems(Startup, load_tree_model)
            .add_systems(Update, chunk_system);
    }
}

pub fn chunk_system(
    mut commands: Commands,
    mut loaded: ResMut<LoadedChunks>,
    player_query: Query<&GlobalTransform, With<Camera3d>>,
    tree_model: Res<TreeModel>, 
    noise: Res<NoiseGenerators>,
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
            let mesh = calc_to_generate_chunk(*coord, &noise);

            let ent = commands.spawn((
                Mesh3d(meshes.add(mesh)), 
                MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                Transform::from_xyz(
                    coord.x as f32 * CHUNK_SIZE as f32 * VERTEX_SPACING,
                    0.0,
                    coord.z as f32 * CHUNK_SIZE as f32 * VERTEX_SPACING,
                ),
            )).id();

            let chunk_x = coord.x as f32 * CHUNK_SIZE as f32 * VERTEX_SPACING;
            let chunk_z = coord.z as f32 * CHUNK_SIZE as f32 * VERTEX_SPACING;
            // load trees and other stuff
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {

                    let wx = chunk_x + x as f32 * VERTEX_SPACING;
                    let wz = chunk_z + z as f32 * VERTEX_SPACING;

                    let local_x = wx - chunk_x;
                    let local_z = wz - chunk_z;


                    if should_tree_spawn(wx as f64, wz as f64, &noise) {
                        let h = get_height(wx as f64, wz as f64, &noise);

                        spawn_tree(
                            &mut commands,
                            &tree_model,
                            ent,
                            local_x,
                            local_z,
                            h,
                        );
                    }
                }
            }
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

#[derive(Resource)]
pub struct TreeModel {
    pub handle: Handle<Scene>,
}

fn load_tree_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let tree: Handle<Scene> = asset_server.load("snowy_pine_tree.glb#Scene0");

    commands.insert_resource(TreeModel { handle: tree });
}

fn spawn_tree(
    commands: &mut Commands,
    tree_model: &TreeModel,
    parent: Entity,
    x: f32,
    z: f32,
    height: f32,
) {
    commands.entity(parent).with_children(|parent| {
        parent.spawn((
            SceneRoot(tree_model.handle.clone()),
            Transform::from_xyz(x, height, z).with_scale(Vec3::splat(1.25)),
        ));
    });
}
