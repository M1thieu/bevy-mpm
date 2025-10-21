//! Grid-to-Particle (G2P) transfer operations
//!
//! Transfers velocities and velocity gradients from grid nodes back to particles.
//! Updates particle positions and deformation state.

use bevy::prelude::*;

use crate::core::{Grid, GRID_RESOLUTION, GridInterpolation, MpmState};
use crate::materials::MaterialModel;

/// Native coordinate-based G2P transfer (eliminates linear index conversions)
pub fn grid_to_particle(
    time: Res<Time>,
    mut state: ResMut<MpmState>,
) {
    let grid_ptr: *const Grid = state.grid() as *const Grid;
    let grid = unsafe { &*grid_ptr };

    let cell_width = grid.cell_width();
    let inv_d = 4.0 / (cell_width * cell_width);

    let particles = state.particles_mut();

    for particle in particles.iter_mut() {
        particle.velocity = Vec2::ZERO;

        // Native coordinate-based interpolation (MLS formulation)
        let interp = GridInterpolation::compute_for_particle(particle.position);

        let mut velocity_gradient = Mat2::ZERO;

        for (coord, weight, cell_distance) in interp.iter_neighbors() {
            if let Some(cell) = grid.get_cell_coord(coord) {
                let weighted_velocity = cell.velocity * weight;

                let outer = Mat2::from_cols(
                    Vec2::new(
                        weighted_velocity.x * cell_distance.x,
                        weighted_velocity.y * cell_distance.x,
                    ),
                    Vec2::new(
                        weighted_velocity.x * cell_distance.y,
                        weighted_velocity.y * cell_distance.y,
                    ),
                );

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
