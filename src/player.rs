use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use std::f32::consts::FRAC_PI_2;
use bevy::core_pipeline::bloom::Bloom;

use crate::noise::NoiseGenerators;
use crate::chunks::get_height;
use crate::chunks::get_surface_normal;
use crate::world_gen::*;

#[derive(Component)]
pub struct FlyCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
    pub speed: f32,
    pub velocity: Vec3,
    pub grounded: bool,
    pub flying: bool,
    pub sledding: bool,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            sensitivity: 0.002,
            speed: 10.0,
            velocity: Vec3::ZERO,
            grounded: true,
            flying: false,
            sledding: false
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, grab_cursor, load_slead))
            .add_systems(Update, (camera_movement, camera_look, handle_input, move_snowballs, sledding_system));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        FlyCamera::default(),
        Bloom::NATURAL, 
        DistanceFog {
                color: Color::srgb(0.8, 0.9, 1.0),
                falloff: FogFalloff::Linear {
                    start: (RENDER_DISTANCE as f32) * CHUNK_SIZE as f32,
                    end: (RENDER_DISTANCE as f32 + 10.0) * CHUNK_SIZE as f32,
                },
                ..default()
        },
        Transform::from_xyz(0.0, 1.5, 5.0),
    ));
    
}

fn grab_cursor(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn camera_look(
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
) {
    let Ok((mut transform, mut camera)) = query.single_mut() else { return };

    for ev in motion.read() {
        camera.yaw   -= ev.delta.x * camera.sensitivity;
        camera.pitch -= ev.delta.y * camera.sensitivity;

        // Clamp pitch
        camera.pitch = camera.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

        // Build two rotations: yaw → world, pitch → local X
        let yaw_rot   = Quat::from_rotation_y(camera.yaw);
        let pitch_rot = Quat::from_rotation_x(camera.pitch);

        // Combine them
        transform.rotation = yaw_rot * pitch_rot;
    }
}

fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    noise: Res<NoiseGenerators>,  
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
) {
    let Ok((mut transform, mut camera)) = query.single_mut() else {
        return;
    };
    // check if sledding
    if camera.sledding {
        return;
    }

    // toggle flight
    if keyboard.just_pressed(KeyCode::KeyF) {
        camera.flying = !camera.flying;
        camera.velocity = Vec3::ZERO;
        if camera.flying {
            camera.grounded = false;
        }
    }

    let dt = time.delta_secs();
    let mut direction = Vec3::ZERO;

    let forward = *transform.forward();
    let right = *transform.right();

    if camera.flying {
        // flying
        if keyboard.pressed(KeyCode::KeyW) { direction += forward; }
        if keyboard.pressed(KeyCode::KeyS) { direction -= forward; }
        if keyboard.pressed(KeyCode::KeyA) { direction -= right; }
        if keyboard.pressed(KeyCode::KeyD) { direction += right; }
        
        if keyboard.pressed(KeyCode::Space) { direction += Vec3::Y; }
        if keyboard.pressed(KeyCode::ShiftLeft) { direction -= Vec3::Y; }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }
        
        transform.translation += direction * camera.speed * 3.0 * dt; // * 3.0 to make flying fastern than walking
    } else {
        // walking
        let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        if keyboard.pressed(KeyCode::KeyW) { direction += forward_flat; }
        if keyboard.pressed(KeyCode::KeyS) { direction -= forward_flat; }
        if keyboard.pressed(KeyCode::KeyA) { direction -= right_flat; }
        if keyboard.pressed(KeyCode::KeyD) { direction += right_flat; }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        if keyboard.pressed(KeyCode::KeyQ) {
            transform.translation += direction * camera.speed * dt * 2.0;
        }
        else {
            transform.translation += direction * camera.speed * dt
        }

        let gravity = -25.0;

        if camera.grounded && keyboard.just_pressed(KeyCode::Space) {
            camera.velocity.y = 8.0;
            camera.grounded = false;
        }

        camera.velocity.y += gravity * dt;
        transform.translation.y += camera.velocity.y * dt;

        let terrain_h = get_height(
            transform.translation.x as f64,
            transform.translation.z as f64,
            &noise,
        );

        let player_height = 1.0;
        let ground_y = terrain_h + player_height;

        if transform.translation.y <= ground_y {
            transform.translation.y = ground_y;
            camera.velocity.y = 0.0;
            camera.grounded = true;
        }
    }
}

// snowball logic

#[derive(Component)]
pub struct Snowball {
    pub velocity: Vec3,
}

fn spawn_snowball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cam: &Transform,
) {
    let pos = cam.translation + (*cam.forward()) * 1.0;
    let vel = *cam.forward() * 20.0;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.1).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        Snowball { velocity: vel },
        Transform::from_translation(pos),
    ));
}

fn move_snowballs(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Snowball)>,
    noise: Res<NoiseGenerators>,
    mut commands: Commands
) {
    let dt = time.delta_secs();
    let gravity = 7.5;

    for (entity, mut t, mut ball) in query.iter_mut() {
        t.translation += ball.velocity * dt;
        ball.velocity.y -= gravity * dt;

        // get height of ground
        let terrain_h = get_height(
            t.translation.x as f64,
            t.translation.z as f64,
            &noise,
        );
        // despawn if on ground
        if t.translation.y <= terrain_h {
            commands.entity(entity).despawn();
            continue;
        }
    }
}

fn handle_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<SledEntity>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cam_state_query: Query<&mut FlyCamera, With<Camera3d>>,
    sled_entity_query: Query<Entity, With<SledEntity>>,
    sled: Res<Sled>,
) {

    if mouse.just_pressed(MouseButton::Left) {
        let Ok(cam) = camera_query.single() else { return };
        spawn_snowball(
            &mut commands,
            &mut meshes,
            &mut materials,
            cam,
        );
    }

    if mouse.just_pressed(MouseButton::Right) {
        let Ok(cam_transform) = camera_query.single_mut() else { return };
        let Ok(mut cam_state) = cam_state_query.single_mut() else { return };

        if !cam_state.sledding {
            spawn_sled(&mut commands, &*cam_transform, &sled, &mut cam_state);
        } else {
            if let Ok(sled_entity) = sled_entity_query.single() {
                commands.entity(sled_entity).despawn();
            }
            cam_state.sledding = false;
        }
    }
}

#[derive(Resource)]
pub struct Sled {
    pub handle: Handle<Scene>,
}

#[derive(Component)]
pub struct SledEntity;

fn load_slead(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let sled: Handle<Scene> = asset_server.load("sled.glb#Scene0");

    commands.insert_resource(Sled { handle: sled });

    println!("loaded candycane")
}

fn spawn_sled(
    commands: &mut Commands,
    cam_transform: &Transform,
    sled: &Sled,
    cam_state: &mut FlyCamera,
) {
    let spawn_pos = cam_transform.translation - Vec3::new(0.0, 1.75, 0.0);

    let sled_rotation = Quat::from_rotation_y(cam_state.yaw);

    commands.spawn((
        SledEntity,
        SledMotion::default(),
        SceneRoot(sled.handle.clone()),
        Transform {
            translation: spawn_pos,
            rotation: sled_rotation,
            scale: Vec3::splat(0.5),
        },
    ));

    cam_state.sledding = true;
}

#[derive(Component, Default)]
pub struct SledMotion {
    pub velocity: Vec3,
}

// helper function, so the camera sticks to the sled
fn sledding_system(
    time: Res<Time>,
    noise: Res<NoiseGenerators>,
    mut cam_q: Query<(&mut Transform, &mut FlyCamera), (With<Camera3d>, Without<SledEntity>)>,
    mut sled_q: Query<(&mut Transform, &mut SledMotion), (With<SledEntity>, Without<Camera3d>)>,
) {
    let Ok((mut cam_t, mut cam_state)) = cam_q.single_mut() else { return };

    if !cam_state.sledding {
        return;
    }

    let Ok((mut sled_t, mut motion)) = sled_q.single_mut() else {
        // sled got despawned but state wasn't reset
        cam_state.sledding = false;
        return;
    };

    let dt = time.delta_secs();

    let gravity = Vec3::new(0.0, -25.0, 0.0); // match your "walking gravity" feel
    let wx = sled_t.translation.x as f64;
    let wz = sled_t.translation.z as f64;

    let normal = get_surface_normal(wx, wz, &noise);
    let accel = acceleration_on_slope(normal, gravity);

    // optional damping so it doesn't accelerate forever
    let damping = 0.9999_f32;
    motion.velocity *= damping;

    motion.velocity += accel * dt;
    sled_t.translation += motion.velocity * dt * 5.0;

    // keep sled on terrain
    let terrain_h = get_height(sled_t.translation.x as f64, sled_t.translation.z as f64, &noise);
    sled_t.translation.y = terrain_h; // tweak if your sled needs an offset above ground

    // stick camera to sled
    cam_t.translation = sled_t.translation + Vec3::new(0.0, 1.75, 0.0);
}

fn acceleration_on_slope(normal: Vec3, gravity: Vec3) -> Vec3 {
    let n = normal.normalize();
    gravity - n * gravity.dot(n) // gravity projected into the slope plane
}