use bevy::prelude::*;

use crate::config::SolverParams;
use crate::materials::utils;
use crate::math::{Real, Vector};

use super::grid::{apply_boundary_conditions, BoundaryHandling, Grid};
use super::particle::Particle;
use super::particle_set::ParticleSet;

#[derive(Resource, Default)]
pub struct ParticleRemap {
    pub map: Vec<Option<usize>>,
}

/// Aggregate simulation state mirroring Sparkl's resource layout.
#[derive(Resource)]
pub struct MpmState {
    particle_set: ParticleSet,
    grid: Grid,
    solver_params: SolverParams,
    gravity: Vector,
    boundary: BoundaryHandling,
}

impl MpmState {
    pub fn new(solver_params: SolverParams, gravity: Vector) -> Self {
        Self {
            particle_set: ParticleSet::new(),
            grid: Grid::new(),
            solver_params,
            gravity,
            boundary: BoundaryHandling::Slip,
        }
    }

    pub fn particle_set(&self) -> &ParticleSet {
        &self.particle_set
    }

    pub fn particle_set_mut(&mut self) -> &mut ParticleSet {
        &mut self.particle_set
    }

    pub fn particle_count(&self) -> usize {
        self.particle_set.len()
    }

    pub fn particles(&self) -> &[Particle] {
        self.particle_set.particles()
    }

    pub fn particles_mut(&mut self) -> &mut [Particle] {
        self.particle_set.particles_mut()
    }

    pub fn add_particle(&mut self, particle: Particle) -> usize {
        self.particle_set.push(particle)
    }

    pub fn rebuild_particle_bins(&mut self) {
        let cell_width = self.grid.cell_width();
        self.particle_set.rebuild_bins(cell_width);
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn solver_params(&self) -> &SolverParams {
        &self.solver_params
    }

    pub fn solver_params_mut(&mut self) -> &mut SolverParams {
        &mut self.solver_params
    }

    pub fn gravity(&self) -> Vector {
        self.gravity
    }

    pub fn set_gravity(&mut self, gravity: Vector) {
        self.gravity = gravity;
    }

    pub fn boundary_mode(&self) -> BoundaryHandling {
        self.boundary
    }

    pub fn set_boundary_mode(&mut self, boundary: BoundaryHandling) {
        self.boundary = boundary;
    }

    pub fn zero_grid(&mut self) {
        self.grid.zero_active_cells();
    }

    pub fn cleanup_grid(&mut self) {
        self.grid.cleanup_empty_cells();
    }

    pub fn integrate_grid_velocities(&mut self, dt: Real) {
        let gravity_step = self.gravity * dt;
        for (coords, node) in self.grid.iter_active_cells_mut() {
            if node.mass > 0.0 {
                let momentum = node.momentum + node.mass * gravity_step;
                let inv_mass = utils::inv_exact(node.mass);
                node.velocity = momentum * inv_mass;

                let coord = IVec2::new(coords.0, coords.1);
                apply_boundary_conditions(node, coord, self.boundary);
            }
        }
    }

    pub fn remove_failed_particles(&mut self) -> Vec<Option<usize>> {
        let mapping = self.particle_set.remove_failed();
        if mapping.is_empty() {
            return mapping;
        }

        self.particle_set.order.clear();
        self.particle_set.regions.clear();
        self.particle_set.active_regions.clear();
        self.particle_set.active_cells.clear();
        self.particle_set.particle_bins.clear();

        self.rebuild_particle_bins();
        mapping
    }
}

pub fn zero_grid(mut state: ResMut<MpmState>) {
    state.zero_grid();
}

pub fn cleanup_grid_cells(mut state: ResMut<MpmState>) {
    state.cleanup_grid();
}

pub fn remove_failed_particles_system(
    mut state: ResMut<MpmState>,
    mut remap: ResMut<ParticleRemap>,
) {
    remap.map = state.remove_failed_particles();
}

pub fn clear_particle_remap_system(mut remap: ResMut<ParticleRemap>) {
    if !remap.map.is_empty() {
        remap.map.clear();
    }
}
