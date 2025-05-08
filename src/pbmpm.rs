use bevy::prelude::*;
use crate::solver::Particle;
use crate::constraints::{ConstraintSolver, IncompressibilityConstraint};
use crate::PbmpmConfig;

/// Run multiple iterations of PBMPM constraint solving
pub fn solve_constraints_pbmpm(
    mut query: Query<&mut Particle>,
    config: Res<PbmpmConfig>,
) {
    // Create our constraint solvers
    let incompressibility_solver = IncompressibilityConstraint;
    
    // First iteration: blend the current deformation with previous solution
    query.par_iter_mut().for_each(|mut particle| {
        // Initial blend between current deformation and previous frame's solution
        let blend_factor = config.warm_start_blend_factor;
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
            
            // Apply constraints using our analytical solver
            let _residual = incompressibility_solver.solve(
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