pub mod constants;
pub mod constraints;
pub mod grid;
pub mod simulation;
pub mod solver;
pub mod pbmpm;

use bevy::prelude::*;

#[derive(Resource)]
pub struct PbmpmConfig {
    pub iteration_count: u32,
    pub relaxation_factor: f32,
    pub warm_start_blend_factor: f32,
}

impl Default for PbmpmConfig {
    fn default() -> Self {
        Self {
            iteration_count: 2,  // Start with a small number of iterations
            relaxation_factor: 0.5,  // Slightly reduced for more stability
            warm_start_blend_factor: 0.3,
        }
    }
}