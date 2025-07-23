use bevy::prelude::*;

use crate::simulation::MaterialType;
use crate::grid::GRID_RESOLUTION;

#[derive(Component)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub affine_momentum_matrix: Mat2,
    pub material_type: MaterialType,
    pub grid_index: u32,
    
    // Particle health system
    pub failed: bool,           // Mark particle for removal
    pub condition_number: f32,  // Numerical stability measure
}

impl Particle {
    pub fn zeroed(material_type: MaterialType) -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0,
            affine_momentum_matrix: Mat2::ZERO,
            material_type,
            grid_index: 0,
            failed: false,
            condition_number: 1.0,
        }
    }

    #[inline(always)]
    pub fn calculate_grid_index(&self) -> u32 {
        let grid_x = (self.position.x as u32).min(GRID_RESOLUTION as u32 - 1);
        let grid_y = (self.position.y as u32).min(GRID_RESOLUTION as u32 - 1);
        grid_y * GRID_RESOLUTION as u32 + grid_x
    }

    /// Check if particle should be marked as failed due to numerical instability
    #[inline(always)]
    pub fn update_health(&mut self) {
        // Check for invalid matrix elements first
        if !self.affine_momentum_matrix.is_finite() {
            self.failed = true;
            self.condition_number = f32::INFINITY;
            return;
        }
        
        // Calculate condition number approximation using matrix norms
        let det = self.affine_momentum_matrix.determinant().abs();
        let trace = (self.affine_momentum_matrix.col(0).x + self.affine_momentum_matrix.col(1).y).abs();
        
        // Better condition number approximation: ratio of largest to smallest singular values
        // For 2x2 matrix, this is approximately |trace| / |det| when det is non-zero
        self.condition_number = if det > 1e-12 {
            trace / det
        } else {
            f32::INFINITY
        };
        
        // Mark particle as failed if numerically unstable
        const CONDITION_THRESHOLD: f32 = 1e6;
        if self.condition_number > CONDITION_THRESHOLD || !self.condition_number.is_finite() {
            self.failed = true;
        }
        
        // Also check for invalid position/velocity/mass
        if !self.position.is_finite() || !self.velocity.is_finite() || !self.mass.is_finite() || self.mass <= 0.0 {
            self.failed = true;
        }
    }
}

// System to update grid indices for spatial sorting
pub fn update_particle_grid_indices(mut particles: Query<&mut Particle>) {
    particles.par_iter_mut().for_each(|mut particle| {
        particle.grid_index = particle.calculate_grid_index();
    });
}

// System to update particle health and mark failed particles
pub fn update_particle_health(mut particles: Query<&mut Particle>) {
    particles.par_iter_mut().for_each(|mut particle| {
        particle.update_health();
    });
}

// System to remove failed particles from the simulation
pub fn cleanup_failed_particles(
    mut commands: Commands,
    particles: Query<(Entity, &Particle)>,
) {
    for (entity, particle) in particles.iter() {
        if particle.failed {
            commands.entity(entity).despawn();
        }
    }
}