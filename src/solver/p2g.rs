//! Particle-to-Grid (P2G) transfer operations
//!
//! Transfers mass, momentum, and forces from particles to grid nodes.
//! Includes stress calculation and MLS affine momentum transfer.

use bevy::prelude::*;

use crate::core::{GridInterpolation, MpmState};
use crate::materials::MaterialModel;
use crate::materials::utils;

/// Two-pass P2G: accumulate mass/momentum, then apply stress forces
/// Identical behavior to the previous split functions, just consolidated
pub fn particle_to_grid(time: Res<Time>, mut state: ResMut<MpmState>) {
    state.rebuild_particle_bins();
    let solver_params = state.solver_params().clone();
    let dt = time.delta_secs();

    let (particle_ptr, particle_len) = {
        let slice = state.particles();
        (slice.as_ptr(), slice.len())
    };
    let particles = unsafe { std::slice::from_raw_parts(particle_ptr, particle_len) };
    let grid = state.grid_mut();
    let cell_width = grid.cell_width();
    let inv_d = 4.0 / (cell_width * cell_width);

    // Pass 1: accumulate mass
    for particle in particles.iter() {
        let interp = GridInterpolation::compute_for_particle(particle.position);

        for (coord, weight, _) in interp.iter_neighbors() {
            let cell = grid.get_cell_coord_mut(coord);
            cell.mass += weight * particle.mass;
        }
    }

    // Pass 2: scatter momentum with stress contribution
    for particle in particles.iter() {
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

        // Affine term (APIC) incorporating stress (Jiang et al. 2015)
        let affine = particle.mass * particle.velocity_gradient - (volume * inv_d * dt) * stress;
        let momentum = particle.mass * particle.velocity;

        for (coord, weight, cell_distance) in interp.iter_neighbors() {
            let cell = grid.get_cell_coord_mut(coord);
            let contribution = affine * cell_distance + momentum;
            cell.momentum += weight * contribution;
        }
    }
}
