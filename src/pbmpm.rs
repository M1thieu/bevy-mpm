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
            // Create a copy of the deformation displacement to work with
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
}