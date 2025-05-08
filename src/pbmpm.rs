use bevy::prelude::*;
use crate::solver::Particle;
use crate::constraints::{ConstraintSolver, IncompressibilityConstraint};
use crate::PbmpmConfig;

/// Run multiple iterations of PBMPM constraint solving with weighted warm starting
/// and trait-based constraint system for extensibility
pub fn solve_constraints_pbmpm(
    mut query: Query<&mut Particle>,
    config: Res<PbmpmConfig>,
) {
    // Create our constraint solvers
    let incompressibility_solver = IncompressibilityConstraint;
    
    // Track convergence across iterations
    let mut max_errors = Vec::with_capacity(config.iteration_count as usize);
    
    // Run multiple iterations of constraint solving
    for iteration in 0..config.iteration_count {
        // Calculate adaptive relaxation factor based on iteration
        let adaptive_relaxation = config.relaxation_factor * 
            (1.0 - (iteration as f32 / config.iteration_count as f32) * 0.3);
        
        // Use thread-local storage for errors and find max afterward
        let error_values = std::sync::Mutex::new(Vec::new());
        
        // Solve constraints for all particles
        query.par_iter_mut().for_each(|mut particle| {
            // Calculate weighted blend of previous solution and current deformation
            let warm_start_weight = if iteration == 0 {
                // Use full configured weight for first iteration
                config.warm_start_weight 
            } else {
                // For subsequent iterations, no warm starting from previous frame
                0.0
            };
            
            // Apply constraints using our analytical solver
            let mut deformation = 
                particle.deformation_displacement * (1.0 - warm_start_weight) +
                particle.prev_deformation_displacement * warm_start_weight;
            
            // Apply constraints and track error
            let error = incompressibility_solver.solve(
                &mut particle,
                &mut deformation,
                adaptive_relaxation
            );
            
            // Update the particle with constrained values
            particle.deformation_displacement = deformation;
            particle.affine_momentum_matrix = deformation;
            
            // Add error to the shared list
            if error > 0.0 {
                error_values.lock().unwrap().push(error);
            }
        });
        
        // Find the maximum error from all threads
        let max_error = error_values.lock().unwrap().iter().fold(0.0f32, |max, &err| max.max(err));
        max_errors.push(max_error);
        
        // Early termination if we've converged enough
        if iteration > 0 && max_error < 1e-4 {
            break;
        }
    }
    
    // After all iterations, store current solution for next frame with adaptive warm starting
    query.par_iter_mut().for_each(|mut particle| {
        // Calculate convergence quality
        let convergence_quality = if max_errors.len() > 1 && max_errors[0] > 1e-5 {
            (max_errors[0] - max_errors[max_errors.len() - 1]).max(0.0) / 
            max_errors[0]
        } else {
            0.5 // Default if we only had one iteration
        };
        
        // Apply warm starting directly proportional to convergence quality
        // This is cleaner and more intuitive - well-converged areas get more warm starting
        let adaptive_weight = config.warm_start_weight * convergence_quality;
        
        particle.prev_deformation_displacement = particle.deformation_displacement * adaptive_weight;
    });
}