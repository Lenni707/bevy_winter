use noise::{Perlin, Seedable};
use bevy::prelude::*;

#[derive(Resource)]
pub struct NoiseGenerators {
    pub height: Perlin,
    pub biome: Perlin,
    pub river: Perlin,
}

impl NoiseGenerators {
    pub fn new(seed: u32) -> Self {
        Self {
            height: Perlin::new().set_seed(seed), 
            biome: Perlin::new().set_seed(seed + 69),
            river: Perlin::new().set_seed(seed + 420),
        }
    }
}