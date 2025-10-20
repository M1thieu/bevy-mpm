//! Particle-to-Grid (P2G) transfer operations
//!
//! Transfers mass, momentum, and forces from particles to grid nodes.
//! Includes stress calculation and MLS affine momentum transfer.

use bevy::prelude::*;

use crate::config::SolverParams;
use crate::core::Particle;
use crate::core::{Grid, GridInterpolation};
use crate::materials::MaterialModel;
use crate::materials::utils;

/// Two-pass P2G: accumulate mass/momentum, then apply stress forces
/// Identical behavior to the previous split functions, just consolidated
pub fn particle_to_grid(
    time: Res<Time>,
    solver_params: Res<SolverParams>,
    particles: Query<&Particle>,
    mut grid: ResMut<Grid>,
) {
    // Pass 1: accumulate mass and APIC momentum
    for particle in &particles {
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

    // Pass 2: apply stress forces
    for particle in &particles {
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
        let stress = particle
            .material_type
            .compute_stress(particle, density, &solver_params);

        // MLS-MPM force application (Jiang et al. 2015, Eq. 16) with quadratic basis scaling
        let eq_16_term_0 = -4.0 * volume * stress * time.delta_secs();

        for (coord, weight, cell_distance) in interp.iter_neighbors() {
            let cell = grid.get_cell_coord_mut(coord);
            let momentum = eq_16_term_0 * weight * cell_distance;
            cell.velocity += momentum;
        }
    }
}
