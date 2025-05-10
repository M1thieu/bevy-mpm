use bevy::prelude::*;
use crate::solver::Particle;
use crate::constraints::{ConstraintSolver, IncompressibilityConstraint};
use crate::bukkit::BukkitSystem;
use crate::PbmpmConfig;

/// Run multiple iterations of PBMPM constraint solving with weighted warm starting
/// and trait-based constraint system for extensibility
pub fn solve_constraints_pbmpm(
    mut query: Query<&mut Particle>,
    config: Res<PbmpmConfig>,
    bukkits: Res<BukkitSystem>
) {
    // Create our constraint solvers
    let incompressibility_solver = IncompressibilityConstraint;
    
    // Clone data to avoid borrowing issues
    let thread_data = bukkits.thread_data.clone();
    let particle_indices = bukkits.particle_indices.clone();
    
    // Run multiple iterations of constraint solving
    for iteration in 0..config.iteration_count {
        // Calculate adaptive warm start weight based on iteration progress
        let warm_start_weight = if iteration == 0 {
            // Use full configured weight for first iteration
            config.warm_start_weight 
        } else {
            // For subsequent iterations, no warm starting from previous frame
            // as we're now working within the current frame's solving process
            0.0
        };
        
        // Process particles by bukkit for better cache locality
        for bukkit_data in &thread_data {
            let bukkit_idx = bukkit_data.bukkit_index;
            
            // Get all particles in this bukkit
            for &entity in &particle_indices[bukkit_idx] {
                if let Ok(mut particle) = query.get_mut(entity) {
                    // Calculate weighted blend of previous solution and current deformation
                    let mut deformation = 
                        particle.deformation_displacement * (1.0 - warm_start_weight) +
                        particle.prev_deformation_displacement * warm_start_weight;
                    
                    // Apply constraints using our analytical solver
                    incompressibility_solver.solve(
                        &mut particle,
                        &mut deformation,
                        config.relaxation_factor
                    );
                    
                    // Update the particle with constrained values
                    particle.deformation_displacement = deformation;
                    particle.affine_momentum_matrix = deformation;
                }
            }
        }
    }
    
    // After all iterations are done, store current solution for next frame
    for bukkit_data in &thread_data {
        let bukkit_idx = bukkit_data.bukkit_index;
        
        for &entity in &particle_indices[bukkit_idx] {
            if let Ok(mut particle) = query.get_mut(entity) {
                particle.prev_deformation_displacement = particle.deformation_displacement;
            }
        }
    }
}