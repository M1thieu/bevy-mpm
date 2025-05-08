use bevy::prelude::*;
use crate::solver::Particle;
use crate::constraints::*;
use crate::PbmpmConfig;

/// Run multiple iterations of PBMPM constraint solving with weighted warm starting
pub fn solve_constraints_pbmpm(
    mut query: Query<&mut Particle>,
    config: Res<PbmpmConfig>,
) {
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
        
        query.par_iter_mut().for_each(|mut particle| {
            // Calculate weighted blend of previous solution and current deformation
            let mut deformation = 
                particle.deformation_displacement * (1.0 - warm_start_weight) +
                particle.prev_deformation_displacement * warm_start_weight;
            
            // Apply constraints with the configured relaxation factor
            solve_incompressibility_constraint(
                &mut particle,
                &mut deformation,
                config.relaxation_factor
            );
            
            // Update the particle with constrained values
            particle.deformation_displacement = deformation;
            particle.affine_momentum_matrix = deformation;
        });
    }
    
    // After all iterations are done, store current solution for next frame
    query.par_iter_mut().for_each(|mut particle| {
        particle.prev_deformation_displacement = particle.deformation_displacement;
    });
}