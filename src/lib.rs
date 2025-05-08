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
    pub warm_start_weight: f32, // Added warm start weight parameter
}

impl Default for PbmpmConfig {
    fn default() -> Self {
        Self {
            iteration_count: 2,
            relaxation_factor: 0.5,
            warm_start_weight: 0.8, // Default to 80% weight from previous solution
        }
    }
}