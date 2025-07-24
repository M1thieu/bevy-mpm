use bevy::prelude::*;

use crate::grid::{GRID_RESOLUTION, Grid, calculate_grid_weights};
use crate::particle::Particle;

/// Implements proper affine matrix update using outer product
pub fn grid_to_particle(time: Res<Time>, mut query: Query<&mut Particle>, grid: Res<Grid>) {
    query.par_iter_mut().for_each(|mut particle| {
        particle.velocity = Vec2::ZERO;

        let (cell_index, weights) = calculate_grid_weights(particle.position);

        let mut b = Mat2::ZERO;

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);

                // Fixed indexing: y * width + x for row-major order
                let cell_index =
                    cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;

                let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;

                if let Some(cell) = grid.cells.get(cell_index) {
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

        let particle_velocity = particle.velocity;

        particle.position += particle_velocity * time.delta_secs();

        // Prevent particles from going out of bounds
        particle.position = particle
            .position
            .clamp(Vec2::splat(1.0), Vec2::splat(GRID_RESOLUTION as f32 - 2.0));
    });
}
