//! Grid-to-Particle (G2P) transfer operations
//!
//! Transfers velocities and velocity gradients from grid nodes back to particles.
//! Updates particle positions and deformation state.

use bevy::prelude::*;

use crate::core::{GRID_RESOLUTION, MpmState, kernel::inv_d};
use crate::materials::MaterialModel;
use crate::math::outer_product;

/// Native coordinate-based G2P transfer (eliminates linear index conversions)
pub fn grid_to_particle(time: Res<Time>, mut state: ResMut<MpmState>) {
    let bins_owned = state.particle_bins().to_owned();
    let (grid, particles, transfer_cache) = state.grid_and_particles_mut_cache();
    let bins = &bins_owned;
    let cell_width = grid.cell_width();
    let inv_d = inv_d(cell_width);
    const COLOUR_COUNT: u8 = 4;

    for colour in 0..COLOUR_COUNT {
        for bin in bins.iter().filter(|b| b.colour == colour) {
            for i in 0..bin.len as usize {
                let idx = bin.indices[i];
                if idx == usize::MAX {
                    continue;
                }
                let particle = &mut particles[idx];
                let transfer = &transfer_cache[idx];

                particle.velocity = Vec2::ZERO;
                let mut velocity_gradient = Mat2::ZERO;

                for &(coord, weight, cell_distance) in &transfer.neighbors {
                    if let Some(cell) = grid.get_cell_coord(coord) {
                        let weighted_velocity = cell.velocity * weight;
                        let outer = outer_product(weighted_velocity, cell_distance);

                        particle.velocity += weighted_velocity;
                        velocity_gradient += outer * (weight * inv_d);
                    }
                }

                particle.affine_momentum_matrix = velocity_gradient;
                particle.velocity_gradient = velocity_gradient;

                // Update deformation gradient: F_new = (I + dt * C) * F_old
                let dt = time.delta_secs();
                let deformation_update = Mat2::IDENTITY + velocity_gradient * dt;
                particle.deformation_gradient = deformation_update * particle.deformation_gradient;

                let material = particle.material_type.clone();
                material.project_deformation(particle);

                let particle_velocity = particle.velocity;

                particle.position += particle_velocity * time.delta_secs();

                // Prevent particles from going out of bounds
                particle.position = particle
                    .position
                    .clamp(Vec2::splat(1.0), Vec2::splat(GRID_RESOLUTION as f32 - 2.0));
            }
        }
    }
}
