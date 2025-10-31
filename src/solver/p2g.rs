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

    let (grid, particles, cache) = state.grid_mut_and_particles_cache();
    let cell_width = grid.cell_width();
    let inv_d = inv_d(cell_width);

    // Pass 1: accumulate mass
    for (idx, particle) in particles.iter().enumerate() {
        let transfer = &cache[idx];
        for &(coord, weight, _) in &transfer.neighbors {
            let cell = grid.get_cell_coord_mut(coord);
            let mass_delta = weight * particle.mass;
            cell.mass += mass_delta;
            cell.fluids.mass += mass_delta;
        }
    }

    // Pass 2: scatter momentum with stress contribution
    // OPTIMIZATION: Batch neighbor cell lookups to avoid double HashMap access
    for (idx, particle) in particles.iter().enumerate() {
        let transfer = &cache[idx];

        // Fetch all 9 neighbor cells ONCE and cache them
        // This avoids 18 HashMap lookups (9 for density + 9 for momentum scatter)
        let mut neighbor_cells: [Option<(f32, Vec2)>; 9] = [None; 9];
        let mut density = 0.0;

        for (i, &(coord, weight, cell_distance)) in transfer.neighbors.iter().enumerate() {
            if let Some(cell) = grid.get_cell_coord(coord) {
                density += cell.mass * weight;
                neighbor_cells[i] = Some((weight, cell_distance));
            }
        }

        // Calculate stress based on material type
        let stress = particle.material_type.compute_stress(particle, density, &solver_params);

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

        // Now scatter momentum using the SAME neighbor iteration
        // We only do ONE HashMap lookup per neighbor instead of two
        for (i, &(coord, _weight, _cell_distance)) in transfer.neighbors.iter().enumerate() {
            if let Some((weight, cell_distance)) = neighbor_cells[i] {
                let cell = grid.get_cell_coord_mut(coord);
                let contribution = affine * cell_distance + momentum;
                let momentum_delta = weight * contribution;
                cell.momentum += momentum_delta;
                cell.fluids.momentum += momentum_delta;

                if psi_mass > 0.0 {
                    let psi_mass_delta = weight * psi_mass;
                    let psi_momentum_delta = weight * psi_momentum;
                    cell.psi_mass += psi_mass_delta;
                    cell.psi_momentum += psi_momentum_delta;
                    cell.fluids.psi_mass += psi_mass_delta;
                    cell.fluids.psi_momentum += psi_momentum_delta;
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
