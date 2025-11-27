use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting); // , spawn_ground
        // Wireframes
        app.add_systems(Update, toggle_wireframe);
        app.add_plugins(WireframePlugin::default());
        // spawn background
        app.insert_resource(ClearColor(Color::srgb_u8(173, 216, 230)));
    }
}

fn setup_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(40.0, 80.0, 40.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Mit F3 Taste togglen
    if keyboard.just_pressed(KeyCode::F3) {
        wireframe_config.global = !wireframe_config.global;
        println!("Wireframe: {}", wireframe_config.global);
    }
}


// fn spawn_ground(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     commands.spawn((
//         Mesh3d(meshes.add(Plane3d::default().mesh().size(2000.0, 2000.0))),
//         MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
//         Transform::default(),
//     ));
// }

