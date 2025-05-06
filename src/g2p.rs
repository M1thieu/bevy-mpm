use bevy::prelude::*;

use crate::grid::{Grid, GRID_RESOLUTION, calculate_grid_weights};
use crate::particle::Particle;

pub fn grid_to_particle(
    time: Res<Time>,
    mut query: Query<&mut Particle>,
    grid: Res<Grid>
) {
    query.par_iter_mut()
        .for_each(|mut particle| {
            // Capture all initial states to avoid borrowing conflicts
            let initial_position = particle.position;
            let _initial_velocity = particle.velocity;

            // Intermediate variables to build new state
            let mut new_velocity = Vec2::ZERO;
            let mut new_affine_momentum = Mat2::ZERO;
            let mut new_position = initial_position;

            let (cell_index, weights) = calculate_grid_weights(initial_position);

            for gx in 0..3 {
                for gy in 0..3 {
                    let weight = weights[gx].x * weights[gy].y;

                    let cell_position =
                        UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    
                    let cell_index =
                        cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;
                    
                    let cell_distance = cell_position.as_vec2() - initial_position + 0.5;
                    let cell_velocity = grid.cells.get(cell_index).unwrap().velocity;
                    let weighted_velocity = cell_velocity * weight;

                    let term = Mat2::from_cols(
                        weighted_velocity * cell_distance.x, 
                        weighted_velocity * cell_distance.y
                    );

                    new_affine_momentum += term;
                    new_velocity += weighted_velocity;
                }
            }

            // Update all particle properties at once
            new_position += new_velocity * time.delta_secs();

            // More sophisticated boundary handling
            let wall_min = 1.0;
            let wall_max = GRID_RESOLUTION as f32 - 2.0;

            // Predictive boundary velocity cap
            let dt_multiplier = 0.1 / time.delta_secs();
            let position_next = new_position + new_velocity * time.delta_secs() * dt_multiplier;
            
            // Adaptive velocity adjustment near boundaries
            if position_next.x < wall_min {
                new_velocity.x += wall_min - position_next.x;
            }
            if position_next.x > wall_max {
                new_velocity.x += wall_max - position_next.x;
            }
            if position_next.y < wall_min {
                new_velocity.y += wall_min - position_next.y;
            }
            if position_next.y > wall_max {
                new_velocity.y += wall_max - position_next.y;
            }

            // Final position clamping
            new_position = new_position.clamp(
                Vec2::splat(wall_min),
                Vec2::splat(wall_max)
            );

            // Apply updates to particle
            particle.velocity = new_velocity;
            particle.affine_momentum_matrix = new_affine_momentum * 4.0;
            particle.position = new_position;
        });
}