use bevy::prelude::*;
use crate::solver::Particle;
use crate::constraints::solve_incompressibility_constraint;
use crate::PbmpmConfig;

/// Run multiple iterations of PBMPM constraint solving
pub fn solve_constraints_pbmpm(
    mut query: Query<&mut Particle>,
    config: Res<PbmpmConfig>,
) {
    // First iteration: blend the current deformation with previous solution
    query.par_iter_mut().for_each(|mut particle| {
        // Initial blend between current deformation and previous frame's solution
        // Start with weight of 0.3 for current and 0.7 for previous solution
        let blend_factor = 0.3;
        let blended_deformation = 
            particle.deformation_displacement * blend_factor + 
            particle.prev_deformation_displacement * (1.0 - blend_factor);
        
        // Use this blended solution as our starting point
        particle.deformation_displacement = blended_deformation;
        particle.affine_momentum_matrix = blended_deformation;
    });

    // Run constraint solving iterations
    for _ in 0..config.iteration_count {
        query.par_iter_mut().for_each(|mut particle| {
            // Make a copy of the current deformation
            let mut deformation = particle.deformation_displacement;
            
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