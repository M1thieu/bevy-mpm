//! Particle-to-Grid (P2G) transfer operations
//!
//! Transfers mass, momentum, and forces from particles to grid nodes.
//! Includes stress calculation and MLS affine momentum transfer.

use bevy::prelude::*;

use crate::config::SolverParams;
use crate::core::Particle;
use crate::core::{Grid, GridInterpolation};
use crate::materials;
use crate::materials::MaterialType;
use crate::materials::utils;

pub fn particle_to_grid_mass_velocity(query: Query<&Particle>, mut grid: ResMut<Grid>) {
    for particle in &query {
        // MLS interpolation using the native coordinate structure
        let interp = GridInterpolation::compute_for_particle(particle.position);

        for (coord, weight, cell_distance) in interp.iter_neighbors() {
            let affine_contrib = particle.affine_momentum_matrix * cell_distance;
            let mass_contribution = weight * particle.mass;

            let cell = grid.get_cell_coord_mut(coord);
            cell.mass += mass_contribution;
            cell.velocity += mass_contribution * (particle.velocity + affine_contrib);
        }
    }
}

pub fn particle_to_grid_forces(
    time: Res<Time>,
    solver_params: Res<SolverParams>,
    mut particles: Query<&mut Particle>,
    mut grid: ResMut<Grid>,
) {
    for particle in &mut particles {
        // Native coordinate-based interpolation
        let interp = GridInterpolation::compute_for_particle(particle.position);

        // Density calculation with direct coordinate access
        let mut density = 0.0;
        for (coord, weight, _) in interp.iter_neighbors() {
            if let Some(cell) = grid.get_cell_coord(coord) {
                density += cell.mass * weight;
            }
        }

        let volume = particle.mass * utils::inv_exact(density);

        // Calculate stress based on material type
        let stress = match &particle.material_type {
            MaterialType::Water => materials::fluids::water::calculate_stress(
                &particle,
                density,
                solver_params.volume_correction_strength,
                solver_params.preserve_fluid_volume && particle.material_type.is_fluid(),
                solver_params.dynamic_viscosity,
            ),
        };

        // MLS-MPM force application (Jiang et al. 2015, Eq. 16) with quadratic basis scaling
        let eq_16_term_0 = -4.0 * volume * stress * time.delta_secs();

        for (coord, weight, cell_distance) in interp.iter_neighbors() {
            let cell = grid.get_cell_coord_mut(coord);
            let momentum = eq_16_term_0 * weight * cell_distance;
            cell.velocity += momentum;
        }
    }
}
