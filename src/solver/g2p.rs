use bevy::prelude::*;
use crate::grid::{Grid, GRID_RESOLUTION, calculate_grid_weights, iter_quadratic_weights, get_grid_cell};
use crate::solver::Particle;
use crate::bukkit::BukkitSystem;
use std::time::Instant;

pub fn grid_to_particle(
    time: Res<Time>,
    mut query: Query<&mut Particle>,
    grid: Res<Grid>,
    mut bukkits: ResMut<BukkitSystem>
) {
    let start = Instant::now();
    
    // Clone data to avoid borrowing issues
    let thread_data = bukkits.thread_data.clone();
    let particle_indices = bukkits.particle_indices.clone();
    
    // Process particles by bukkit for better cache locality
    for bukkit_data in &thread_data {
        let bukkit_idx = bukkit_data.bukkit_index;
        
        // Get all particles in this bukkit
        for &entity in &particle_indices[bukkit_idx] {
            if let Ok(mut particle) = query.get_mut(entity) {
                let position = particle.position;
                
                // Reset velocity
                particle.velocity = Vec2::ZERO;
                
                let (cell_index, weights) = calculate_grid_weights(position);

                // Gather grid information
                let mut velocity_sum = Vec2::ZERO;
                let mut deformation_matrix = Mat2::ZERO;

                // NEW: Use iter_quadratic_weights and combine processing + marking
                for (gx, gy, weight) in iter_quadratic_weights(&weights) {
                    let cell_position = UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    
                    // NEW: Use cell_in_bukkit_range instead of explicit checks
                    if !bukkit_data.cell_in_bukkit_range(cell_position) {
                        continue;
                    }
                    
                    // NEW: Use get_grid_cell - single call, process AND mark
                    if let Some((cell_idx, cell)) = get_grid_cell(&grid, cell_position) {
                        let cell_distance = (cell_position.as_vec2() - position) + 0.5;
                        let weighted_velocity = cell.velocity * weight;

                        let term = Mat2::from_cols(
                            weighted_velocity * cell_distance.x, 
                            weighted_velocity * cell_distance.y
                        );
                        
                        deformation_matrix += term;
                        velocity_sum += weighted_velocity;
                        
                        // Mark cell as active immediately (no second lookup!)
                        bukkits.mark_grid_cell_active(cell_idx);
                    }
                }

                // Scale the deformation matrix
                deformation_matrix *= 4.0;
                
                // Update particle
                particle.deformation_displacement = deformation_matrix;
                particle.affine_momentum_matrix = deformation_matrix;
                particle.velocity = velocity_sum;
                
                // Update position
                particle.position += velocity_sum * time.delta_secs();
                
                // Apply boundary constraint
                particle.position = particle.position.clamp(
                    Vec2::splat(1.0),
                    Vec2::splat(GRID_RESOLUTION as f32 - 2.0)
                );
            }
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("g2p: {:.3}ms", elapsed);
}