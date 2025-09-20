//! Grid-to-Particle (G2P) transfer operations
//!
//! Transfers velocities and velocity gradients from grid nodes back to particles.
//! Updates particle positions and deformation state.

use bevy::prelude::*;

use crate::core::Particle;
use crate::core::{GRID_RESOLUTION, Grid, GridInterpolation};

/// Implements proper affine matrix update using outer product
pub fn grid_to_particle(time: Res<Time>, mut query: Query<&mut Particle>, grid: Res<Grid>) {
    for mut particle in &mut query {
        particle.velocity = Vec2::ZERO;

        // Unified interpolation in G2P
        let interp = GridInterpolation::compute_for_particle(particle.position);

        let mut b = Mat2::ZERO;

        for (neighbor_idx, (&neighbor_linear_index, &cell_distance)) in
            interp.neighbor_indices.iter().zip(&interp.cell_distances).enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let weight = interp.weight_for_neighbor(neighbor_idx);

                if let Some(cell) = grid.cells.get(linear_index) {
                    let weighted_velocity = cell.velocity * weight;

                    let term = Mat2::from_cols(
                        Vec2::new(
                            weighted_velocity.x * cell_distance.x,
                            weighted_velocity.y * cell_distance.x,
                        ),
                        Vec2::new(
                            weighted_velocity.x * cell_distance.y,
                            weighted_velocity.y * cell_distance.y,
                        ),
                    );

                    b += term;
                    particle.velocity += weighted_velocity;
                }
            }
        }

        particle.affine_momentum_matrix = b;
        particle.velocity_gradient = b; // Store velocity gradient for P2G APIC

        // Update deformation gradient: F_new = (I + dt * velocity_gradient) * F_old
        let dt = time.delta_secs();
        let velocity_gradient = b;
        let deformation_update = Mat2::IDENTITY + velocity_gradient * dt;
        particle.deformation_gradient = deformation_update * particle.deformation_gradient;

        let particle_velocity = particle.velocity;

        particle.position += particle_velocity * time.delta_secs();

        // Prevent particles from going out of bounds
        particle.position = particle
            .position
            .clamp(Vec2::splat(1.0), Vec2::splat(GRID_RESOLUTION as f32 - 2.0));
    }
}
