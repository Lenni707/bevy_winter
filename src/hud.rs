use bevy::prelude::*;
use noise::NoiseFn;

use crate::chunks::*;
use crate::world_gen::*;
use crate::noise::NoiseGenerators;

// Component for HUD Text
#[derive(Component)]
struct HudText;

// FPS counter resource
#[derive(Resource, Default)]
struct FpsCounter {
    frames: u32,
    timer: f32,
    fps: f32,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FpsCounter>()
            .add_systems(Startup, setup_hud)
            .add_systems(Update, (update_fps, update_hud).chain());
    }
}

fn setup_hud(mut commands: Commands) {
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(10.0),
        padding: UiRect::all(Val::Px(10.0)),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new("FPS: 0\nPos: 0.0 0.0 0.0\nChunk: 0 0\nBiome: None"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            HudText,
        ));
    });
}

fn update_fps(time: Res<Time>, mut fps: ResMut<FpsCounter>) {
    fps.frames += 1;
    fps.timer += time.delta_secs();

    if fps.timer >= 1.0 {
        fps.fps = fps.frames as f32 / fps.timer;
        fps.frames = 0;
        fps.timer = 0.0;
    }
}

// --- SIMPLE biome detection (same rules as terrain) ---
pub fn detect_biome(x: f32, z: f32, noise: &NoiseGenerators) -> Biome {
    let bx: f64 = x as f64 * BIOME_FREQ;
    let bz: f64 = z as f64 * BIOME_FREQ;

    let biome_val: f32 = noise.biome.get([bx, bz]) as f32;

    if biome_val < -0.2 {
        return Biome::Plains
    } else{
        return Biome::Forest
    };
}

fn update_hud(
    fps: Res<FpsCounter>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    noise: Res<NoiseGenerators>,
    mut query: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = query.single_mut() else { return };
    let Ok(transform) = camera_query.single() else { return };

    let pos = transform.translation();

    // Convert world → chunk coordinates
    let cx = (pos.x / (CHUNK_SIZE as f32 * VERTEX_SPACING)).floor() as i32;
    let cz = (pos.z / (CHUNK_SIZE as f32 * VERTEX_SPACING)).floor() as i32;

    let biome = detect_biome(pos.x, pos.z, &noise);

    // Update HUD text
    **text = format!(
        "FPS: {:.0}\nPos: {:.1} {:.1} {:.1}\nChunk: {} {}\nBiome: {:?}",
        fps.fps,
        pos.x, pos.y, pos.z,
        cx, cz,
        biome
    );
}
