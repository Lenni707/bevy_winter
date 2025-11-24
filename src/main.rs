use bevy::prelude::*;

use crate::player::PlayerPlugin;
use crate::world::WorldPlugin;
use crate::snowflake::SnowflakePlugin;
use crate::world_gen::WorldGenPlugin;
use crate::hud::HudPlugin;

mod player;
mod world;
mod snowflake;
mod world_gen;
mod chunks;
mod hud;
mod noise;

fn main() {
    App::new()
        .insert_resource(noise::NoiseGenerators::new(12345))
        .add_plugins((DefaultPlugins, PlayerPlugin, WorldPlugin, SnowflakePlugin, WorldGenPlugin, HudPlugin))
        .add_systems(Update, exit_on_esc)
        .run();
}

fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
