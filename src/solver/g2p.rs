use bevy::prelude::*;

use crate::grid::{GRID_RESOLUTION, Grid, calculate_grid_weights, get_neighbor_indices};
use crate::particle::Particle;

/// Implements proper affine matrix update using outer product
pub fn grid_to_particle(time: Res<Time>, mut query: Query<&mut Particle>, grid: Res<Grid>) {
    // Sort particles by grid cell for better cache performance
    let mut particles: Vec<_> = query.iter_mut().collect();
    particles.sort_by_key(|particle| particle.grid_index);

    for mut particle in particles {
        particle.velocity = Vec2::ZERO;

        let (cell_index, weights) = calculate_grid_weights(particle.position);
        let center_linear_index = cell_index.y as usize * 128 + cell_index.x as usize;
        let neighbor_indices = get_neighbor_indices(center_linear_index);

        // Pre-compute cell distances (cache optimization)
        let mut cell_distances = [Vec2::ZERO; 9];
        for neighbor_idx in 0..9 {
            let gx = neighbor_idx % 3;
            let gy = neighbor_idx / 3;
            let cell_position = UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
            cell_distances[neighbor_idx] = (cell_position.as_vec2() - particle.position) + 0.5;
        }

        let mut b = Mat2::ZERO;

        for (neighbor_idx, &neighbor_linear_index) in neighbor_indices.iter().enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let gx = neighbor_idx % 3;
                let gy = neighbor_idx / 3;
                let weight = weights[gx].x * weights[gy].y;

                let cell_distance = cell_distances[neighbor_idx];  // Use pre-computed distance
                
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

        let particle_velocity = particle.velocity;

        particle.position += particle_velocity * time.delta_secs();

        // Prevent particles from going out of bounds
        particle.position = particle
            .position
            .clamp(Vec2::splat(1.0), Vec2::splat(GRID_RESOLUTION as f32 - 2.0));
    }
}
