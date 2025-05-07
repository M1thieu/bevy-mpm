use bevy::prelude::*;
use crate::particle::Particle;
use crate::constraints::solve_incompressibility_constraint;
use crate::PbmpmConfig;

/// Run multiple iterations of PBMPM constraint solving
pub fn solve_constraints_pbmpm(
    mut query: Query<&mut Particle>,
    config: Res<PbmpmConfig>,
) {
    // Run multiple iterations of constraint solving
    for _ in 0..config.iteration_count {
        query.par_iter_mut().for_each(|mut particle| {
            // Use previous solution as starting point instead of current deformation
            // This is the key change for warm starting
            let mut deformation = particle.prev_deformation_displacement;
            
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