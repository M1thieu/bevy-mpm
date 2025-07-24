use bevy::prelude::*;

use crate::constants::{DYNAMIC_VISCOSITY, EOS_POWER, EOS_STIFFNESS, REST_DENSITY};
use crate::grid::{Grid, calculate_grid_weights, safe_inverse, safe_grid_index};
use crate::particle::Particle;
use crate::simulation::MaterialType;
use crate::solver_params::SolverParams;

pub fn particle_to_grid_mass_velocity(query: Query<&Particle>, mut grid: ResMut<Grid>) {
    // Sort particles by grid cell for better cache performance
    let mut particles: Vec<&Particle> = query.iter().collect();
    particles.sort_by_key(|particle| particle.grid_index);

    for particle in particles {
        let (cell_index, weights) = calculate_grid_weights(particle.position);

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;
                let q = particle.affine_momentum_matrix * cell_distance;

                let mass_contribution = weight * particle.mass;

                // Safer bounds checking with early exit
                if let Some(linear_index) = safe_grid_index(cell_position) {
                    if let Some(cell) = grid.cells.get_mut(linear_index) {
                        cell.mass += mass_contribution;
                        cell.velocity += mass_contribution * (particle.velocity + q);
                    }
                }
            }
        }
    }
}

pub fn particle_to_grid_forces(
    time: Res<Time>,
    solver_params: Res<SolverParams>,
    mut particles: Query<&mut Particle>,
    mut grid: ResMut<Grid>,
) {
    // Sort particles by grid cell for better cache performance
    let mut particle_refs: Vec<_> = particles.iter_mut().collect();
    particle_refs.sort_by_key(|particle| particle.grid_index);

    for particle in particle_refs {
        let (cell_index, weights) = calculate_grid_weights(particle.position);

        let mut density = 0.0;

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);

                // Safer bounds checking  
                if let Some(linear_index) = safe_grid_index(cell_position) {
                    if let Some(cell) = grid.cells.get(linear_index) {
                        density += cell.mass * weight;
                    }
                }
            }
        }

        let volume = particle.mass * safe_inverse(density);

        // Calculate stress based on material type
        let stress = match &particle.material_type {
            MaterialType::Water {
                vp0: _,
                ap: _,
                jp: _,
            } => {
                // Original EOS pressure
                let eos_pressure = f32::max(
                    -0.1,
                    EOS_STIFFNESS * ((density / REST_DENSITY).powi(EOS_POWER as i32) - 1.0),
                );

                // Volume preservation correction (parameter-driven)
                let volume_correction =
                    if solver_params.preserve_fluid_volume && particle.material_type.is_fluid() {
                        // Simple volume deviation correction
                        let current_volume = volume;
                        let target_volume = particle.volume0;
                        let volume_deviation = (current_volume - target_volume) / target_volume;

                        // Apply correction proportional to deviation
                        solver_params.volume_correction_strength * volume_deviation * REST_DENSITY
                    } else {
                        0.0
                    };

                // Combined pressure (EOS + volume preservation)
                let total_pressure = eos_pressure + volume_correction;
                let stress = Mat2::IDENTITY * -total_pressure;

                let dudv = particle.affine_momentum_matrix;
                let mut strain = dudv;
                let trace = strain.col(1).x + strain.col(0).y;
                strain.col_mut(0).y = trace;
                strain.col_mut(1).x = trace;
                let viscosity_term = DYNAMIC_VISCOSITY * strain;

                stress + viscosity_term
            }
        };

        let eq_16_term_0 = -volume * stress * time.delta_secs();

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;

                // Safer bounds checking
                if let Some(linear_index) = safe_grid_index(cell_position) {
                    if let Some(cell) = grid.cells.get_mut(linear_index) {
                        let momentum = eq_16_term_0 * weight * cell_distance;
                        cell.velocity += momentum;
                    }
                }
            }
        }
    }
}
