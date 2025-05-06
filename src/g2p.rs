use bevy::prelude::*;
use crate::grid::{Grid, GRID_RESOLUTION, calculate_grid_weights};
use crate::particle::Particle;
use crate::simulation::MaterialType;
use crate::constraints::solve_incompressibility_constraint;

pub fn grid_to_particle(
    time: Res<Time>,
    mut query: Query<&mut Particle>,
    grid: Res<Grid>
) {
    query.par_iter_mut()
        .for_each(|mut particle| {
            // Store position in a local variable to avoid borrow issues
            let position = particle.position;
            
            // Reset velocity
            particle.velocity = Vec2::ZERO;
            
            let (cell_index, weights) = calculate_grid_weights(position);

            // Gather grid information
            let mut velocity_sum = Vec2::ZERO;
            let mut deformation_matrix = Mat2::ZERO;

            for gx in 0..3 {
                for gy in 0..3 {
                    let weight = weights[gx].x * weights[gy].y;
                    let cell_position = UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    let cell_index = cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;

                    if cell_index < grid.cells.len() {
                        let cell_distance = (cell_position.as_vec2() - position) + 0.5;
                        let weighted_velocity = grid.cells[cell_index].velocity * weight;

                        let term = Mat2::from_cols(
                            weighted_velocity * cell_distance.x, 
                            weighted_velocity * cell_distance.y
                        );
                        
                        deformation_matrix += term;
                        velocity_sum += weighted_velocity;
                    }
                }
            }

            // Scale the deformation matrix
            deformation_matrix *= 4.0;
            
            // Apply PBMPM constraint (moved to the constraints module)
            let mut constrained_matrix = deformation_matrix;
            solve_incompressibility_constraint(&mut particle, &mut constrained_matrix, 0.5);
            
            // Update particle
            particle.deformation_displacement = constrained_matrix;
            particle.affine_momentum_matrix = constrained_matrix;
            particle.velocity = velocity_sum;
            
            // Update position
            particle.position += velocity_sum * time.delta_secs();
            
            // Apply boundary constraint
            particle.position = particle.position.clamp(
                Vec2::splat(1.0),
                Vec2::splat(GRID_RESOLUTION as f32 - 2.0)
            );
        });
}