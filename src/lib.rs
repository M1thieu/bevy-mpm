use bevy::prelude::*;

pub mod constants;
pub mod grid;
pub mod materials;
pub mod particle;
pub mod simulation;
pub mod solver;
pub mod solver_params;

// Public re-exports for clean API
pub use grid::{Cell, GRID_RESOLUTION, Grid};
pub use particle::Particle;
pub use simulation::MaterialType;
pub use solver_params::SolverParams;

use crate::constants::GRAVITY;
use crate::grid::{calculate_grid_velocities, zero_grid};
use crate::particle::{
    cleanup_failed_particles, update_particle_grid_indices, update_particle_health,
};
use crate::solver::{grid_to_particle, particle_to_grid_forces, particle_to_grid_mass_velocity};

pub struct MpmPlugin;

impl Plugin for MpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid {
            cells: vec![Cell::zeroed(); GRID_RESOLUTION * GRID_RESOLUTION],
        })
        .insert_resource(SolverParams::default())
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
            )
                .chain(),
        );
    }
}

fn calculate_grid_velocities_with_gravity(time: Res<Time>, grid: ResMut<Grid>) {
    calculate_grid_velocities(time, grid, GRAVITY);
}
