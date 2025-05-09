pub mod constants;
pub mod constraints;
pub mod grid;
pub mod simulation;
pub mod solver;
pub mod pbmpm;
pub mod bukkit; 

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use grid::{Grid, calculate_grid_velocities};
use solver::prelude::*;
use constants::GRAVITY;

#[derive(Resource, Clone)]
pub struct PbmpmConfig {
    pub iteration_count: u32,
    pub relaxation_factor: f32,
    pub warm_start_weight: f32,
}

impl Default for PbmpmConfig {
    fn default() -> Self {
        Self {
            iteration_count: 2,  // Start with a small number of iterations
            relaxation_factor: 0.5,  // Slightly reduced for more stability
            warm_start_weight: 0.2, // Default to 12.5% weight from previous solution
                                     // NOTE: This allows a natural view of particles individually above 0.5 makes all too cohesive
        }
    }
}

/// A plugin that sets up the PBMPM simulation systems in the correct order
pub struct PbmpmPlugin {
    /// The simulation configuration
    pub config: PbmpmConfig,
}

impl Default for PbmpmPlugin {
    fn default() -> Self {
        Self {
            config: PbmpmConfig::default(),
        }
    }
}

/// Initializes the grid with cells
fn init_grid(mut grid: ResMut<Grid>) {
    use grid::GRID_RESOLUTION;
    
    grid.cells.clear();
    grid.cells.reserve_exact(GRID_RESOLUTION * GRID_RESOLUTION);
    for _ in 0..(GRID_RESOLUTION * GRID_RESOLUTION) {
        grid.cells.push(grid::Cell::zeroed());
    }
}

/// Wrapper for the grid velocity calculation that uses the GRAVITY constant
fn calculate_grid_velocities_wrapper(
    time: Res<Time>,
    mut grid: ResMut<Grid>
) {
    calculate_grid_velocities(time, grid, GRAVITY);
}

impl Plugin for PbmpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid { cells: Vec::new() })
           .insert_resource(bukkit::BukkitConfig::default())
           .insert_resource(bukkit::BukkitSystem::new(&bukkit::BukkitConfig::default()))
           .insert_resource(self.config.clone())
           .add_plugins(FrameTimeDiagnosticsPlugin::default())
           .add_systems(Startup, init_grid)
           .add_systems(
               FixedUpdate,
               (
                   bukkit::selective_grid_clear,
                   bukkit::assign_particles_to_bukkits,
                   particle_to_grid_mass_velocity,
                   particle_to_grid_forces,
                   calculate_grid_velocities_wrapper,
                   grid_to_particle,
                   pbmpm::solve_constraints_pbmpm,
               ).chain(),
           );
    }
}