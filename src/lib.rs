use bevy::prelude::*;

pub mod constants;
pub mod grid;
pub mod particle;
pub mod simulation;
pub mod solver;

// Public re-exports for clean API
pub use particle::Particle;
pub use simulation::MaterialType;
pub use grid::{Grid, Cell, GRID_RESOLUTION};

use crate::constants::GRAVITY;
use crate::grid::{zero_grid, calculate_grid_velocities};
use crate::solver::{particle_to_grid_mass_velocity, particle_to_grid_forces, grid_to_particle};
use crate::particle::{update_particle_grid_indices, update_particle_health, cleanup_failed_particles};

pub struct MpmPlugin;

impl Plugin for MpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid {
            cells: vec![Cell::zeroed(); GRID_RESOLUTION * GRID_RESOLUTION],
        })
        .add_systems(
            Update,
            (
                update_particle_grid_indices,
                update_particle_health,
                zero_grid,
                particle_to_grid_mass_velocity,
                particle_to_grid_forces,
                calculate_grid_velocities_with_gravity,
                grid_to_particle,
                cleanup_failed_particles,
            ).chain()
        );
    }
}

fn calculate_grid_velocities_with_gravity(
    time: Res<Time>,
    grid: ResMut<Grid>,
) {
    calculate_grid_velocities(time, grid, GRAVITY);
}