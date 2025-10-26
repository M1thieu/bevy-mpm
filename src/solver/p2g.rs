//! Particle-to-Grid (P2G) transfer operations
//!
//! Transfers mass, momentum, and forces from particles to grid nodes.
//! Includes stress calculation and MLS affine momentum transfer.

use bevy::prelude::*;

use crate::core::{MpmState, kernel::inv_d};
use crate::materials::MaterialModel;
use crate::materials::utils;

/// Two-pass P2G: accumulate mass/momentum, then apply stress forces
/// Identical behavior to the previous split functions, just consolidated
pub fn particle_to_grid(time: Res<Time>, mut state: ResMut<MpmState>) {
    state.rebuild_particle_bins();
    let solver_params = state.solver_params().clone();
    let dt = time.delta_secs();

    let bins_owned = state.particle_bins().to_owned();
    let (grid, particles, cache) = state.grid_mut_and_particles_cache();
    let bins = &bins_owned;
    let cell_width = grid.cell_width();
    let inv_d = inv_d(cell_width);
    const COLOUR_COUNT: u8 = 4;

    // Pass 1: accumulate mass
    for colour in 0..COLOUR_COUNT {
        for bin in bins.iter().filter(|b| b.colour == colour) {
            for i in 0..bin.len as usize {
                let idx = bin.indices[i];
                if idx == usize::MAX {
                    continue;
                }
                let particle = &particles[idx];
                let transfer = &cache[idx];
                for &(coord, weight, _) in &transfer.neighbors {
                    let cell = grid.get_cell_coord_mut(coord);
                    cell.mass += weight * particle.mass;
                }
            }
        }
    }

    // Pass 2: scatter momentum with stress contribution
    for colour in 0..COLOUR_COUNT {
        for bin in bins.iter().filter(|b| b.colour == colour) {
            for i in 0..bin.len as usize {
                let idx = bin.indices[i];
                if idx == usize::MAX {
                    continue;
                }
                let particle = &particles[idx];
                let transfer = &cache[idx];

                // Density calculation with direct coordinate access
                let mut density = 0.0;
                for &(coord, weight, _) in &transfer.neighbors {
                    if let Some(cell) = grid.get_cell_coord(coord) {
                        density += cell.mass * weight;
                    }
                }

                // Calculate stress based on material type
                let stress =
                    particle
                        .material_type
                        .compute_stress(particle, density, &solver_params);

                let psi_mass = if particle.phase > 0.0
                    && particle.crack_propagation_factor != 0.0
                    && !particle.failed
                {
                    particle.mass
                } else {
                    0.0
                };
                let psi_momentum = psi_mass * particle.psi_pos;

                // Affine term (APIC) incorporating stress (Jiang et al. 2015)
                // CRITICAL: Use volume0 (rest volume) not current volume
                let affine = particle.mass * particle.velocity_gradient
                    - (particle.volume0 * inv_d * dt) * stress;
                let momentum = particle.mass * particle.velocity;

                for &(coord, weight, cell_distance) in &transfer.neighbors {
                    let cell = grid.get_cell_coord_mut(coord);
                    let contribution = affine * cell_distance + momentum;
                    cell.momentum += weight * contribution;
                    if psi_mass > 0.0 {
                        cell.psi_mass += weight * psi_mass;
                        cell.psi_momentum += weight * psi_momentum;
                    }
                }
            }
        }
    }

    // Pass 3: Convert momentum to velocity immediately after accumulation
    // This must happen in P2G for correct force computation timing
    for (_, cell) in grid.iter_active_cells_mut() {
        if cell.mass > 0.0 {
            let inv_mass = utils::inv_exact(cell.mass);
            cell.velocity = cell.momentum * inv_mass;
        }
    }
}
