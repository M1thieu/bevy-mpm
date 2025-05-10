use bevy::prelude::*;
use std::time::Instant;

use crate::grid::{Grid, GRID_RESOLUTION, calculate_grid_weights};
use crate::solver::Particle;
use crate::bukkit::BukkitSystem;

// Constants 
const EOS_STIFFNESS: f32 = 10.0;
const EOS_POWER: u8 = 4;
const REST_DENSITY: f32 = 2.0;
const DYNAMIC_VISCOSITY: f32 = 0.1;

pub fn particle_to_grid_mass_velocity(
    query: Query<&Particle>,
    mut grid: ResMut<Grid>,
    mut bukkits: ResMut<BukkitSystem>
) {
    let start = Instant::now();
    
    // Clone both the thread data and particle indices to avoid borrowing issues
    let thread_data = bukkits.thread_data.clone();
    let particle_indices = bukkits.particle_indices.clone();
    
    // Process particles by bukkit for better cache locality
    for bukkit_data in &thread_data {
        let bukkit_idx = bukkit_data.bukkit_index;
        
        // Get all particles in this bukkit
        for &entity in &particle_indices[bukkit_idx] {
            if let Ok(particle) = query.get(entity) {
                let (cell_index, weights) = calculate_grid_weights(particle.position);

                for gx in 0..3 {
                    for gy in 0..3 {
                        let weight = weights[gx].x * weights[gy].y;

                        let cell_position =
                            UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                        
                        // Skip cells outside this bukkit's grid range
                        if cell_position.x < bukkit_data.grid_min_x as u32 || 
                           cell_position.x >= bukkit_data.grid_max_x as u32 ||
                           cell_position.y < bukkit_data.grid_min_y as u32 || 
                           cell_position.y >= bukkit_data.grid_max_y as u32 {
                            continue;
                        }
                        
                        let cell_distance =
                            (cell_position.as_vec2() - particle.position) + 0.5;
                        let q = particle.affine_momentum_matrix * cell_distance;

                        let mass_contribution = weight * particle.mass;

                        // Fixed indexing: y * width + x for row-major order
                        let cell_idx =
                            cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;

                        if let Some(cell) = grid.cells.get_mut(cell_idx) {
                            cell.mass += mass_contribution;
                            cell.velocity += mass_contribution * (particle.velocity + q);
                            
                            // Mark this cell as active
                            bukkits.mark_grid_cell_active(cell_idx);
                        }
                    }
                }
            }
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("p2g_mass: {:.3}ms", elapsed);
}

pub fn particle_to_grid_forces(
    time: Res<Time>,
    query: Query<&Particle>,
    mut grid: ResMut<Grid>,
    mut bukkits: ResMut<BukkitSystem>
) {
    let start = Instant::now();
    
    // Clone both the thread data and particle indices to avoid borrowing issues
    let thread_data = bukkits.thread_data.clone();
    let particle_indices = bukkits.particle_indices.clone();
    
    // Process particles by bukkit for better cache locality
    for bukkit_data in &thread_data {
        let bukkit_idx = bukkit_data.bukkit_index;
        
        // Get all particles in this bukkit
        for &entity in &particle_indices[bukkit_idx] {
            if let Ok(particle) = query.get(entity) {
                let (cell_index, weights) = calculate_grid_weights(particle.position);

                let mut density = 0.0;

                // Density calculation
                for gx in 0..3 {
                    for gy in 0..3 {
                        let weight = weights[gx].x * weights[gy].y;

                        let cell_position =
                            UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                        
                        // Skip cells outside this bukkit's grid range
                        if cell_position.x < bukkit_data.grid_min_x as u32 || 
                           cell_position.x >= bukkit_data.grid_max_x as u32 ||
                           cell_position.y < bukkit_data.grid_min_y as u32 || 
                           cell_position.y >= bukkit_data.grid_max_y as u32 {
                            continue;
                        }

                        // Fixed indexing: y * width + x for row-major order
                        let cell_idx =
                            cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;

                        if let Some(cell) = grid.cells.get(cell_idx) {
                            density += cell.mass * weight;
                        }
                    }
                }

                let volume = particle.mass / density;

                let pressure = f32::max(-0.1, EOS_STIFFNESS * ((density / REST_DENSITY).powi(EOS_POWER as i32) - 1.0));

                let mut stress = Mat2::IDENTITY * -pressure;

                let dudv = particle.affine_momentum_matrix;
                let mut strain = dudv;

                let trace = strain.col(1).x + strain.col(0).y;
                strain.col_mut(0).y = trace;
                strain.col_mut(1).x = trace;

                let viscosity_term = DYNAMIC_VISCOSITY * strain;

                stress += viscosity_term;

                let eq_16_term_0 = -volume * 4.0 * stress * time.delta_secs();

                // Momentum calculation
                for gx in 0..3 {
                    for gy in 0..3 {
                        let weight = weights[gx].x * weights[gy].y;

                        let cell_position =
                            UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                        
                        // Skip cells outside this bukkit's grid range
                        if cell_position.x < bukkit_data.grid_min_x as u32 || 
                           cell_position.x >= bukkit_data.grid_max_x as u32 ||
                           cell_position.y < bukkit_data.grid_min_y as u32 || 
                           cell_position.y >= bukkit_data.grid_max_y as u32 {
                            continue;
                        }
                        
                        let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;

                        // Fixed indexing: y * width + x for row-major order
                        let cell_idx =
                            cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;
                        
                        if let Some(cell) = grid.cells.get_mut(cell_idx) {
                            let momentum = eq_16_term_0 * weight * cell_distance;
                            cell.velocity += momentum;
                            
                            // Mark this cell as active
                            bukkits.mark_grid_cell_active(cell_idx);
                        }
                    }
                }
            }
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("p2g_forces: {:.3}ms", elapsed);
}