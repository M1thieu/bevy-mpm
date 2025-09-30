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

        // Native coordinate-based interpolation (MLS formulation)
        let interp = GridInterpolation::compute_for_particle(particle.position);

        // B matrix from Jiang et al. 2015 (before multiplying with the fixed D^-1 factor)
        let mut b = Mat2::ZERO;

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

        // MLS-MPM affine velocity field: C = 4 * B for quadratic B-spline basis
        let affine_velocity = b * 4.0;
        particle.affine_momentum_matrix = affine_velocity;

        // Store gradient for constitutive models and deformation tracking
        particle.velocity_gradient = affine_velocity;

        // Update deformation gradient: F_new = (I + dt * C) * F_old
        let dt = time.delta_secs();
        let deformation_update = Mat2::IDENTITY + affine_velocity * dt;
        particle.deformation_gradient = deformation_update * particle.deformation_gradient;

        // Project F to hydrostatic for fluids
        // In 2D: F = [[a,0],[0,a]] → det(F) = a², so a = J^0.25 gives det(F) = √J
        if particle.material_type.is_fluid() {
            let jacobian = particle.deformation_gradient.determinant();
            let scale = jacobian.abs().powf(0.25);
            particle.deformation_gradient = Mat2::IDENTITY * scale;
        }

        let particle_velocity = particle.velocity;

        particle.position += particle_velocity * time.delta_secs();

        // Prevent particles from going out of bounds
        particle.position = particle
            .position
            .clamp(Vec2::splat(1.0), Vec2::splat(GRID_RESOLUTION as f32 - 2.0));
    }
}
