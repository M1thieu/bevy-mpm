pub mod constants;
pub mod constraints;
pub mod grid;
pub mod simulation;
pub mod p2g;
pub mod g2p;
pub mod particle;
pub mod pbmpm;

use bevy::prelude::*;

#[derive(Resource)]
pub struct PbmpmConfig {
    pub iteration_count: u32,
    pub relaxation_factor: f32,
}

impl Default for PbmpmConfig {
    fn default() -> Self {
        Self {
            iteration_count: 5,  // Start with a small number of iterations
            relaxation_factor: 0.5,  // Slightly reduced for more stability
        }
    }
}