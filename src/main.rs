use bevy::prelude::*;

// todo:
// -> procedual terrain generation with Noise-Driven Heightmap Terrain

use crate::player::PlayerPlugin;
use crate::world::WorldPlugin;
use crate::snowflake::SnowflakePlugin;
use crate::world_gen::WorldGenPlugin;

mod player;
mod world;
mod snowflake;
mod world_gen;
mod chunks;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PlayerPlugin, WorldPlugin, SnowflakePlugin, WorldGenPlugin))
        .run();
}
