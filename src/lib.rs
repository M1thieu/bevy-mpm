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
pub mod geometry;
pub mod materials;
pub mod math;
pub mod solver;

// Clean public API - everything you need to get started
pub use config::{GRAVITY, REST_DENSITY, SolverParams};
pub use core::{GRID_RESOLUTION, Grid, GridNode, MpmState, Particle, ParticleRemap};
pub use materials::{FluidParams, MaterialType};

use crate::core::update_particles_health;
use crate::core::{
    cleanup_grid_cells, clear_particle_remap_system, remove_failed_particles_system, zero_grid,
};
use crate::solver::{grid_to_particle, grid_update, particle_to_grid};

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
        let params = self
            .solver_params
            .clone()
            .unwrap_or_else(SolverParams::default);
        app.insert_resource(MpmState::new(params, GRAVITY));
        app.insert_resource(ParticleRemap::default());

        app.add_systems(
            Update,
            (
                update_particle_health_system,
                zero_grid,
                particle_to_grid,
                cleanup_grid_cells,
                grid_update,
                grid_to_particle,
                remove_failed_particles_system,
                clear_particle_remap_system,
            )
                .chain(),
        );

        if self.debug {
            info!("MPM debug mode enabled");
        }
    }
}

fn update_particle_health_system(mut state: ResMut<MpmState>) {
    let particles = state.particles_mut();
    update_particles_health(particles);
}
