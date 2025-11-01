//! Grid-to-Particle (G2P) transfer operations
//!
//! Transfers velocities and velocity gradients from grid nodes back to particles.
//! Updates particle positions and deformation state.

use bevy::prelude::*;

use crate::core::{GRID_RESOLUTION, MpmState, kernel::inv_d};
use crate::materials::MaterialModel;
use crate::math::{zero_vector, zero_matrix, identity_matrix, outer_product, from_bevy_vec2};

/// Native coordinate-based G2P transfer (eliminates linear index conversions)
pub fn grid_to_particle(time: Res<Time>, mut state: ResMut<MpmState>) {
    let (grid, particles, transfer_cache) = state.grid_and_particles_mut_cache();
    let cell_width = grid.cell_width();
    let inv_d = inv_d(cell_width);

    // Simple single-threaded G2P (ready for parallelization later)
    for (idx, particle) in particles.iter_mut().enumerate() {
        let transfer = &transfer_cache[idx];

        particle.velocity = zero_vector();
        let mut velocity_gradient = zero_matrix();

        for &(coord, weight, cell_distance) in &transfer.neighbors {
            if let Some(cell) = grid.get_cell_coord(coord) {
                let weighted_velocity = cell.velocity * weight;  // nalgebra Vector
                let cell_dist_na = from_bevy_vec2(cell_distance);
                let outer = outer_product(weighted_velocity, cell_dist_na);

                particle.velocity += weighted_velocity;
                velocity_gradient += outer * (weight * inv_d);
            }
        }

        particle.affine_momentum_matrix = velocity_gradient;
        particle.velocity_gradient = velocity_gradient;

        // Update deformation gradient: F_new = (I + dt * C) * F_old
        let dt = time.delta_secs();
        let deformation_update = identity_matrix() + velocity_gradient * dt;
        particle.deformation_gradient = deformation_update * particle.deformation_gradient;

        let material = particle.material_type.clone();
        material.project_deformation(particle);

        let particle_velocity = particle.velocity;

        particle.position += particle_velocity * time.delta_secs();

        // Prevent particles from going out of bounds
        let min = 1.0;
        let max = GRID_RESOLUTION as f32 - 2.0;
        particle.position.x = particle.position.x.clamp(min, max);
        particle.position.y = particle.position.y.clamp(min, max);
    }
}
