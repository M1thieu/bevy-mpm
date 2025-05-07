use bevy::prelude::*;
use crate::solver::Particle;
use crate::simulation::MaterialType;

/// Solves the incompressibility constraint for liquid particles
/// 
/// This is the core of the Position-Based MPM (PBMPM) approach where we
/// modify the deformation displacement to maintain constant volume
pub fn solve_incompressibility_constraint(
    particle: &mut Particle,
    deformation_displacement: &mut Mat2,
    relaxation_factor: f32
) {
    // Only apply to water materials
    if let MaterialType::Water { .. } = particle.material_type {
        // Calculate deformation trace (volumetric strain)
        let deformation_trace = deformation_displacement.col(0).x + 
                              deformation_displacement.col(1).y;
        
        // Store current density to avoid borrowing issues
        let current_density = particle.liquid_density;
        
        // Update liquid density based on deformation
        // New density = old density * (1 + volumetric strain)
        let new_density = current_density * (1.0 + deformation_trace);
        
        // Safety clamp to avoid instability (from PBMPM paper)
        particle.liquid_density = new_density.max(0.05);
        
        // Calculate hydrostatic constraint
        // This drives the liquid toward incompressibility
        let volume_error = 1.0 / current_density - 1.0 - deformation_trace;
        let hydrostatic_constraint = volume_error * relaxation_factor;
        
        // Apply constraint by modifying the diagonal elements
        // This adds a hydrostatic pressure component to counteract volume changes
        deformation_displacement.col_mut(0).x += hydrostatic_constraint;
        deformation_displacement.col_mut(1).y += hydrostatic_constraint;
    }
}

/// A placeholder for future elastic material constraints
pub fn solve_elastic_constraint(
    _particle: &mut Particle,
    _deformation_displacement: &mut Mat2,
    _relaxation_factor: f32
) {
    // This will be implemented later when we add elastic materials
}