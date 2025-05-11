use bevy::prelude::*;
use crate::solver::Particle;
use crate::simulation::MaterialType;

/// Trait for constraint solvers
/// This allows us to have different constraint implementations for different materials
/// while maintaining a common interface
pub trait ConstraintSolver {
    fn solve(&self, particle: &mut Particle, deformation: &mut Mat2, relaxation_factor: f32) -> f32;
}

/// Implementation of incompressibility constraint with analytical derivatives
#[repr(C)] // GPU memory alignment for future WGPU transition
pub struct IncompressibilityConstraint;

impl ConstraintSolver for IncompressibilityConstraint {
    fn solve(&self, particle: &mut Particle, deformation: &mut Mat2, relaxation_factor: f32) -> f32 {
        // Only apply to water materials
        if let MaterialType::Liquid { .. } = particle.material_type {
            // Calculate deformation trace (volumetric strain)
            let deformation_trace = deformation.col(0).x + deformation.col(1).y;
            
            // Store current density to avoid borrowing issues
            let current_density = particle.liquid_density;
            
            // Update liquid density based on deformation
            // New density = old density * (1 + volumetric strain)
            let new_density = current_density * (1.0 + deformation_trace);
            
            // Safety clamp to avoid instability (from PBMPM paper)
            particle.liquid_density = new_density.max(0.05);
            
            // Calculate volume error - this is our constraint
            let volume_error = 1.0 / current_density - 1.0 - deformation_trace;
            
            // ANALYTICAL DERIVATIVE IMPLEMENTATION:
            // The derivative of the constraint with respect to the deformation matrix
            // For incompressibility in 2D, the gradient is identity matrix [1,0,0,1]
            // This gives us the optimal direction to adjust the deformation to satisfy the constraint
            let lambda = volume_error * relaxation_factor;
            
            // Apply correction directly to diagonal elements
            // This is the optimal correction that minimizes |ΔF|² while satisfying the constraint
            deformation.col_mut(0).x += lambda;
            deformation.col_mut(1).y += lambda;
            
            // Return absolute constraint error for convergence monitoring
            return volume_error.abs();
        }
        
        0.0 // No error for non-water materials
    }
}

/// Placeholder for elastic material constraint
#[repr(C)] // GPU memory alignment for future WGPU transition
pub struct ElasticConstraint;

impl ConstraintSolver for ElasticConstraint {
    fn solve(&self, particle: &mut Particle, deformation: &mut Mat2, relaxation_factor: f32) -> f32 {
        // This will be implemented later when we add elastic materials
        0.0
    }
}

/// Legacy function for backward compatibility
/// Use the trait-based implementation for new code
#[inline]
pub fn solve_incompressibility_constraint(
    particle: &mut Particle,
    deformation_displacement: &mut Mat2,
    relaxation_factor: f32
) -> f32 {
    let constraint = IncompressibilityConstraint;
    constraint.solve(particle, deformation_displacement, relaxation_factor)
}

/// Legacy function for backward compatibility
/// Use the trait-based implementation for new code
#[inline]
pub fn solve_elastic_constraint(
    particle: &mut Particle,
    deformation_displacement: &mut Mat2,
    relaxation_factor: f32,
) -> f32 {
    let constraint = ElasticConstraint;
    constraint.solve(particle, deformation_displacement, relaxation_factor)
}