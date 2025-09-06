//! Particle-to-Grid (P2G) transfer operations
//!
//! Transfers mass, momentum, and forces from particles to grid nodes.
//! Includes stress calculation and APIC momentum transfer.

use bevy::prelude::*;

use crate::config::SolverParams;
use crate::core::Particle;
use crate::core::{
    GRID_RESOLUTION, Grid, calculate_grid_weights,
    get_neighbor_indices, calculate_neighbor_distances,
};
use crate::materials;
use crate::materials::utils;
use crate::materials::MaterialType;

pub fn particle_to_grid_mass_velocity(query: Query<&Particle>, mut grid: ResMut<Grid>) {
    // Sort particles by grid cell for better cache performance
    let mut particles: Vec<&Particle> = query.iter().collect();
    particles.sort_by_key(|particle| particle.grid_index);

    for particle in particles {
        let (cell_index, weights) = calculate_grid_weights(particle.position);
        let center_linear_index = cell_index.y as usize * GRID_RESOLUTION + cell_index.x as usize;
        let neighbor_indices = get_neighbor_indices(center_linear_index);

        // Pre-compute cell distances for all neighbors (cache optimization)
        let cell_distances = calculate_neighbor_distances(particle.position, cell_index);

        for (neighbor_idx, &neighbor_linear_index) in neighbor_indices.iter().enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let gx = neighbor_idx % 3;
                let gy = neighbor_idx / 3;
                let weight = weights[gx].x * weights[gy].y;

                let cell_distance = cell_distances[neighbor_idx];
                let q = particle.affine_momentum_matrix * cell_distance;

                let mass_contribution = weight * particle.mass;

                if let Some(cell) = grid.cells.get_mut(linear_index) {
                    cell.mass += mass_contribution;
                    cell.velocity += mass_contribution * (particle.velocity + q);
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
        let center_linear_index = cell_index.y as usize * GRID_RESOLUTION + cell_index.x as usize;
        let neighbor_indices = get_neighbor_indices(center_linear_index);

        // Pre-compute cell distances for reuse in both loops (cache optimization)
        let cell_distances = calculate_neighbor_distances(particle.position, cell_index);

        let mut density = 0.0;

        for (neighbor_idx, &neighbor_linear_index) in neighbor_indices.iter().enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let gx = neighbor_idx % 3;
                let gy = neighbor_idx / 3;
                let weight = weights[gx].x * weights[gy].y;

                if let Some(cell) = grid.cells.get(linear_index) {
                    density += cell.mass * weight;
                }
            }
        }

        let volume = particle.mass * utils::inv_exact(density);

        // Calculate stress based on material type
        let stress = match &particle.material_type {
            MaterialType::Water { .. } => {
                // Use organized water material function
                materials::fluids::water::calculate_stress(
                    &particle,
                    density,
                    solver_params.volume_correction_strength,
                    solver_params.preserve_fluid_volume && particle.material_type.is_fluid(),
                    solver_params.dynamic_viscosity,
                )
            }
        };

        // Use current working stress-force method for now (keep fluids working)
        let eq_16_term_0 = -volume * stress * time.delta_secs();

        for (neighbor_idx, &neighbor_linear_index) in neighbor_indices.iter().enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let gx = neighbor_idx % 3;
                let gy = neighbor_idx / 3;
                let weight = weights[gx].x * weights[gy].y;

                let cell_distance = cell_distances[neighbor_idx];

                if let Some(cell) = grid.cells.get_mut(linear_index) {
                    // Traditional force-based approach (working for fluids)
                    let momentum = eq_16_term_0 * weight * cell_distance;
                    cell.velocity += momentum;
                }
            }
        }
    }
}
