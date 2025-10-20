//! MLS-MPM (Moving Least Squares Material Point Method) simulation for Bevy
//!
//! ```rust
//! use bevy::prelude::*;
//! use mpm2d::MpmPlugin;
//!
//! App::new().add_plugins((DefaultPlugins, MpmPlugin::default())).run();
//! ```

use bevy::prelude::*;

pub mod config;
pub mod core;
pub mod materials;
pub mod solver;

// Clean public API - everything you need to get started
pub use config::{GRAVITY, SolverParams};
pub use core::{Cell, GRID_RESOLUTION, Grid, Particle};
pub use materials::MaterialType;

use crate::core::{calculate_grid_velocities, cleanup_grid_cells, zero_grid};
use crate::core::{cleanup_failed_particles, update_particle_health};
use crate::solver::{grid_to_particle, particle_to_grid};

pub struct MpmPlugin {
    pub solver_params: Option<SolverParams>,
    pub debug: bool,
}

impl Default for MpmPlugin {
    fn default() -> Self {
        Self {
            solver_params: None,
            debug: false,
        }
    }
}

impl MpmPlugin {
    pub fn with_params(solver_params: SolverParams) -> Self {
        Self {
            solver_params: Some(solver_params),
            debug: false,
        }
    }

    pub fn with_debug() -> Self {
        Self {
            solver_params: None,
            debug: true,
        }
    }
}

impl Plugin for MpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid::new());

        if let Some(params) = &self.solver_params {
            app.insert_resource(params.clone());
        } else {
            app.insert_resource(SolverParams::default());
        }

        app.add_systems(
            Update,
            (
                update_particle_health,
                zero_grid,
                particle_to_grid,
                cleanup_grid_cells,
                calculate_grid_velocities_with_gravity,
                grid_to_particle,
                cleanup_failed_particles,
            )
                .chain(),
        );

        if self.debug {
            info!("MPM debug mode enabled");
        }
    }
}

fn calculate_grid_velocities_with_gravity(time: Res<Time>, grid: ResMut<Grid>) {
    calculate_grid_velocities(time, grid, GRAVITY);
}
