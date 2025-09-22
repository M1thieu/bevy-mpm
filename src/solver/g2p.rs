//! Grid-to-Particle (G2P) transfer operations
//!
//! Transfers velocities and velocity gradients from grid nodes back to particles.
//! Updates particle positions and deformation state.

use bevy::prelude::*;

use crate::core::Particle;
use crate::core::{GRID_RESOLUTION, Grid, GridInterpolation};

/// Native coordinate-based G2P transfer (eliminates linear index conversions)
pub fn grid_to_particle(time: Res<Time>, mut query: Query<&mut Particle>, grid: Res<Grid>) {
    for mut particle in &mut query {
        particle.velocity = Vec2::ZERO;

        // Native coordinate-based interpolation
        let interp = GridInterpolation::compute_for_particle(particle.position);

        let mut b = Mat2::ZERO;

        // Direct coordinate iteration - no conversions anywhere
        for (coord, weight, cell_distance) in interp.iter_neighbors() {
            if let Some(cell) = grid.get_cell_coord(coord) {
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
