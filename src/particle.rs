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
        }
    }

    #[inline(always)]
    pub fn calculate_grid_index(&self) -> u32 {
        let grid_x = (self.position.x as u32).min(GRID_RESOLUTION as u32 - 1);
        let grid_y = (self.position.y as u32).min(GRID_RESOLUTION as u32 - 1);
        grid_y * GRID_RESOLUTION as u32 + grid_x
    }
}

// System to update grid indices for spatial sorting
pub fn update_particle_grid_indices(mut particles: Query<&mut Particle>) {
    particles.par_iter_mut().for_each(|mut particle| {
        particle.grid_index = particle.calculate_grid_index();
    });
}