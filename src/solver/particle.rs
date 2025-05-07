use bevy::prelude::*;
use crate::simulation::MaterialType;

#[derive(Component)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub affine_momentum_matrix: Mat2,
    // New fields for PBMPM
    pub deformation_displacement: Mat2, // Tracks deformation for PBMPM
    pub prev_deformation_displacement: Mat2, // Stores previous frame's solution for warm starting
    pub liquid_density: f32, // Track objective volume/density
    pub material_type: MaterialType,
}

impl Particle {
    pub fn zeroed(material_type: MaterialType) -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0,
            affine_momentum_matrix: Mat2::ZERO,
            deformation_displacement: Mat2::ZERO, // Initialize to zero matrix
            prev_deformation_displacement: Mat2::ZERO, // Initialize to zero matrix
            liquid_density: 1.0, // Initialize to default density
            material_type,
        }
    }
}