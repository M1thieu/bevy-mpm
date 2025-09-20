//! Particle-to-Grid (P2G) transfer operations
//!
//! Transfers mass, momentum, and forces from particles to grid nodes.
//! Includes stress calculation and APIC momentum transfer.

use bevy::prelude::*;

use crate::config::SolverParams;
use crate::core::Particle;
use crate::core::{Grid, GridInterpolation};
use crate::materials;
use crate::materials::utils;
use crate::materials::MaterialType;

pub fn particle_to_grid_mass_velocity(query: Query<&Particle>, mut grid: ResMut<Grid>) {
    for particle in &query {
        // Compute all interpolation data once
        let interp = GridInterpolation::compute_for_particle(particle.position);

        for (neighbor_idx, (&neighbor_linear_index, &cell_distance)) in
            interp.neighbor_indices.iter().zip(&interp.cell_distances).enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let weight = interp.weight_for_neighbor(neighbor_idx);
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
    for particle in &mut particles {
        // Unified interpolation computation
        let interp = GridInterpolation::compute_for_particle(particle.position);

        let mut density = 0.0;

        for (neighbor_idx, &neighbor_linear_index) in interp.neighbor_indices.iter().enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let weight = interp.weight_for_neighbor(neighbor_idx);

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

        for (neighbor_idx, (&neighbor_linear_index, &cell_distance)) in
            interp.neighbor_indices.iter().zip(&interp.cell_distances).enumerate() {
            if let Some(linear_index) = neighbor_linear_index {
                let weight = interp.weight_for_neighbor(neighbor_idx);

                if let Some(cell) = grid.cells.get_mut(linear_index) {
                    // Traditional force-based approach (working for fluids)
                    let momentum = eq_16_term_0 * weight * cell_distance;
                    cell.velocity += momentum;
                }
            }
        }
    }
}
