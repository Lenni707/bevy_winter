use bevy::prelude::*;
use rand::{Rng, thread_rng};
use std::f32::consts::TAU;
use bevy::pbr::NotShadowCaster;

const SNOW_RADIUS: f32 = 40.0;
const SNOW_PER_SECOND: f32 = 2400.0;
const SPAWN_HEIGHT: f32 = 2.0;

use crate::noise::NoiseGenerators;
use crate::chunks::get_height;

pub struct SnowflakePlugin;

impl Plugin for SnowflakePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnowflakeAssets>()
           .add_systems(Startup, setup_assets)
           .add_systems(Update, (spawn_snowflakes, update_snowflakes));
    }
}

#[derive(Component)]
struct Snowflake {
    velocity: Vec3,
    rotation_speed: Vec3,
}

#[derive(Resource, Default)]
struct SnowflakeAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn setup_assets(
    mut assets: ResMut<SnowflakeAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    assets.mesh = meshes.add(Rectangle::new(0.2, 0.2));
    assets.material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("snowflake.png")),
        base_color: Color::linear_rgb(10.0, 10.0, 10.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        ..default()
    }
    );
}

fn spawn_snowflakes(
    mut commands: Commands,
    assets: Res<SnowflakeAssets>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    time: Res<Time>,
) {
    let Ok(cam_transform) = camera_query.single() else { return };
    let cam_pos = cam_transform.translation();
    let mut rng = thread_rng();

    // spawn every second
    let dt = time.delta_secs();
    let to_spawn = (SNOW_PER_SECOND * dt).round() as usize;

    for _ in 0..to_spawn {
        let angle = rng.gen_range(0.0..TAU);
        let radius = SNOW_RADIUS * rng.gen_range(0.0..1.0f32).sqrt(); // größe
        
        let x = cam_pos.x + angle.cos() * radius;
        let z = cam_pos.z + angle.sin() * radius;
        let y = cam_pos.y + SPAWN_HEIGHT + rng.gen_range(2.0..17.0);

        commands.spawn((
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.material.clone()),
            NotShadowCaster,
            Transform::from_xyz(x, y, z),
            Snowflake {
                velocity: Vec3::new(0.0, -rng.gen_range(2.0..4.0), 0.0),
                rotation_speed: Vec3::new(
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-2.0..2.0),
                ),
            },
        ));
    }
}

fn update_snowflakes(
    mut commands: Commands,
    time: Res<Time>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut query: Query<(Entity, &mut Transform, &Snowflake)>,
    noise: Res<NoiseGenerators>,
) {
    let dt = time.delta_secs();
    let cam_pos = camera_query.single().map(|t| t.translation()).unwrap_or(Vec3::ZERO);
    let despawn_dist_sq = (SNOW_RADIUS * 1.2).powi(2);

    for (entity, mut transform, snowflake) in &mut query {
        transform.translation += snowflake.velocity * dt;
        transform.rotate_local_x(snowflake.rotation_speed.x * dt);
        transform.rotate_local_y(snowflake.rotation_speed.y * dt);
        transform.rotate_local_z(snowflake.rotation_speed.z * dt);

        // höhe von terrain ausrechnen
        let terrain_h = get_height(
            transform.translation.x as f64,
            transform.translation.z as f64,
            &noise,
        );
        // despawnen when unter der ausgerechneten höhe
        if transform.translation.y <= terrain_h {
            commands.entity(entity).despawn();
            continue;
        }
        // despawnen, wenn aus reichweite von Player
        if transform.translation.xz().distance_squared(cam_pos.xz()) > despawn_dist_sq {
            commands.entity(entity).despawn();
        }
    }
}
