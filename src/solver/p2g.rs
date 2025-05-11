use std::time::Instant;
use bevy::prelude::*;
use crate::grid::*;
use crate::solver::Particle;
use crate::bukkit::BukkitSystem;
use crate::constants::*;

pub fn particle_to_grid(
    time: Res<Time>,
    query: Query<&Particle>,
    mut grid: ResMut<Grid>,
    mut bukkits: ResMut<BukkitSystem>
) {
    let start = Instant::now();
    
    // Clone both the thread data and particle indices to avoid borrowing issues
    let thread_data = bukkits.thread_data.clone();
    let particle_indices = bukkits.particle_indices.clone();
    
    // FIRST PHASE: Mass and velocity transfer
    for thread_data in &thread_data {
        let bukkit_idx = thread_data.bukkit_index;
        
        for &entity in &particle_indices[bukkit_idx] {
            if let Ok(particle) = query.get(entity) {
                let (cell_index, weights) = grid_calculate_weights(particle.position);

                for (gx, gy, weight) in grid_iter_quadratic_weights(&weights) {
                    let cell_position = UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    
                    if !thread_data.cell_in_bukkit_range(cell_position) {
                        continue;
                    }
                    
                    let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;
                    let q = particle.affine_momentum_matrix * cell_distance;
                    let mass_contribution = weight * particle.mass;
                    
                    if let Some((cell_idx, cell)) = grid_get_cell_mut(&mut grid, cell_position) {
                        cell.mass += mass_contribution;
                        cell.velocity += mass_contribution * (particle.velocity + q);
                        
                        // Mark this cell as active
                        bukkits.mark_grid_cell_active(cell_idx);
                    }
                }
            }
        }
    }
    
    // SECOND PHASE: Forces (using the updated grid masses)
    for thread_data in &thread_data {
        let bukkit_idx = thread_data.bukkit_index;
        
        for &entity in &particle_indices[bukkit_idx] {
            if let Ok(particle) = query.get(entity) {
                let (cell_index, weights) = grid_calculate_weights(particle.position);

                let mut density = 0.0;

                // Density calculation
                for (gx, gy, weight) in grid_iter_quadratic_weights(&weights) {
                    let cell_position = UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    
                    if !thread_data.cell_in_bukkit_range(cell_position) {
                        continue;
                    }

                    if let Some((_cell_idx, cell)) = grid_get_cell(&grid, cell_position) {
                        density += cell.mass * weight;
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

                stress += DYNAMIC_VISCOSITY * strain;
                let eq_16_term_0 = -volume * 4.0 * stress * time.delta_secs();

                // Momentum calculation
                for (gx, gy, weight) in grid_iter_quadratic_weights(&weights) {
                    let cell_position = UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    
                    if !thread_data.cell_in_bukkit_range(cell_position) {
                        continue;
                    }
                    
                    let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;
                    
                    if let Some((cell_idx, cell)) = grid_get_cell_mut(&mut grid, cell_position) {
                        let momentum = eq_16_term_0 * weight * cell_distance;
                        cell.velocity += momentum;
                        
                        // Mark this cell as active
                        bukkits.mark_grid_cell_active(cell_idx);
                    }
                }
            }
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("p2g: {:.3}ms", elapsed);
}