//! Material Point Method simulation for Bevy
//! 
//! MPM physics simulation supporting fluids and solids.
//!
//! ```rust
//! use bevy::prelude::*;
//! use mpm2d::MpmPlugin;
//!
//! App::new().add_plugins((DefaultPlugins, MpmPlugin)).run();
//! ```

use bevy::prelude::*;

pub mod config;
pub mod core;
pub mod materials;
pub mod solver;

// Clean public API - everything you need to get started
pub use config::{SolverParams, GRAVITY};
pub use core::{Cell, GRID_RESOLUTION, Grid, Particle};
pub use materials::MaterialType;

use crate::core::{calculate_grid_velocities, zero_grid};
use crate::core::{
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
