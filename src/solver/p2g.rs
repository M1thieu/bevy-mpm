use bevy::prelude::*;

use crate::constants;
use crate::grid::{Cell, Grid, GRID_RESOLUTION, calculate_grid_weights};
use crate::simulation::MaterialType;
use crate::solver::Particle;

// Constants 
const EOS_STIFFNESS: f32 = 10.0;
const EOS_POWER: u8 = 4;
const REST_DENSITY: f32 = 2.0;
const DYNAMIC_VISCOSITY: f32 = 0.1;

pub fn particle_to_grid_mass_velocity(
    query: Query<&Particle>,
    mut grid: ResMut<Grid>
) {
    for particle in query {
        let (cell_index, weights) = calculate_grid_weights(particle.position);

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance =
                    (cell_position.as_vec2() - particle.position) + 0.5;
                let q = particle.affine_momentum_matrix * cell_distance;

                let mass_contribution = weight * particle.mass;

                // Fixed indexing: y * width + x for row-major order
                let cell_index =
                    cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;

                let cell = grid.cells.get_mut(cell_index).unwrap();

                cell.mass += mass_contribution;

                cell.velocity += mass_contribution * (particle.velocity + q);
            }
        }
    }
}

pub fn particle_to_grid_forces(
    time: Res<Time>,
    query: Query<&Particle>,
    mut grid: ResMut<Grid>
) {
    for particle in query {
        let (cell_index, weights) = calculate_grid_weights(particle.position);

        let mut density = 0.0;

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);

                // Fixed indexing: y * width + x for row-major order
                let cell_index =
                    cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;

                let cell = grid.cells.get_mut(cell_index).unwrap();

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

        let viscosity_term = DYNAMIC_VISCOSITY * strain;

        stress += viscosity_term;

        let eq_16_term_0 = -volume * 4.0 * stress * time.delta_secs();

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;

                // Fixed indexing: y * width + x for row-major order
                let cell_index =
                    cell_position.y as usize * GRID_RESOLUTION + cell_position.x as usize;
                let cell = grid.cells.get_mut(cell_index).unwrap();

                let momentum = eq_16_term_0 * weight * cell_distance;

                cell.velocity += momentum;
            }
        }
    }
}