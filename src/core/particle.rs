//! Material particles for MPM simulation
//!
//! Particles carry position, velocity, mass and material properties.

use crate::materials::MaterialType;
use crate::math::{
    Matrix, Real, Vector, identity_matrix, matrix_determinant, matrix_trace, zero_matrix,
    zero_vector,
};

#[derive(Clone)]
pub struct Particle {
    pub position: Vector,
    pub velocity: Vector,
    pub mass: Real,
    pub volume0: Real,
    pub radius0: Real,
    pub affine_momentum_matrix: Matrix, // MLS affine velocity field (C matrix)
    pub velocity_gradient: Matrix,
    pub deformation_gradient: Matrix,
    pub material_type: MaterialType,

    // Simulation bookkeeping
    pub grid_index: u64,
    pub phase: Real,
    pub psi_pos: Real,
    pub is_static: bool,
    pub kinematic_velocity: Option<Vector>,

    // Health tracking
    pub failed: bool,
    pub condition_number: Real,
}

impl Particle {
    pub fn zeroed(material_type: MaterialType) -> Self {
        Self {
            position: zero_vector(),
            velocity: zero_vector(),
            mass: 1.0,
            volume0: 1.0,
            radius0: 1.0,
            affine_momentum_matrix: zero_matrix(),
            velocity_gradient: zero_matrix(),
            deformation_gradient: identity_matrix(),
            material_type,
            grid_index: 0,
            phase: 1.0,
            psi_pos: 0.0,
            is_static: false,
            kinematic_velocity: None,
            failed: false,
            condition_number: 1.0,
        }
    }

    pub fn new(position: Vector, material_type: MaterialType) -> Self {
        Self {
            position,
            material_type,
            ..Self::zeroed(MaterialType::water())
        }
    }

    pub fn with_velocity(mut self, velocity: Vector) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_mass(mut self, mass: Real) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_radius(mut self, radius: Real) -> Self {
        self.radius0 = radius;
        self
    }

    /// Create particle with specific density and radius
    pub fn with_density(radius: Real, density: Real) -> Self {
        let volume = std::f32::consts::PI * radius * radius;
        Self {
            position: zero_vector(),
            velocity: zero_vector(),
            mass: volume * density,
            volume0: volume,
            radius0: radius,
            ..Self::zeroed(MaterialType::water())
        }
    }

    #[inline(always)]
    pub fn current_volume(&self, density: Real) -> Real {
        if density > 0.0 {
            self.mass / density
        } else {
            self.volume0
        }
    }

    #[inline(always)]
    pub fn density_from_volume(&self, volume: Real) -> Real {
        if volume > 0.0 {
            self.mass / volume
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn rest_density(&self) -> Real {
        if self.volume0 > 0.0 {
            self.mass / self.volume0
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn jacobian(&self) -> Real {
        matrix_determinant(&self.deformation_gradient)
    }

    #[inline(always)]
    pub fn current_volume_from_deformation(&self) -> Real {
        let jacobian = self.jacobian();
        self.volume0 * jacobian.abs()
    }

    #[inline(always)]
    pub fn update_health(&mut self) {
        if !matrix_is_finite(&self.affine_momentum_matrix) {
            self.failed = true;
            self.condition_number = Real::INFINITY;
            return;
        }

        let det = matrix_determinant(&self.affine_momentum_matrix).abs();
        let trace = matrix_trace(&self.affine_momentum_matrix).abs();

        self.condition_number = if det > 1e-12 {
            trace / det
        } else {
            Real::INFINITY
        };

        const CONDITION_THRESHOLD: Real = 1e6;
        if self.condition_number > CONDITION_THRESHOLD || !self.condition_number.is_finite() {
            self.failed = true;
        }

        if !self.position.is_finite()
            || !self.velocity.is_finite()
            || !self.mass.is_finite()
            || self.mass <= 0.0
        {
            self.failed = true;
        }

        if !self.volume0.is_finite() || self.volume0 <= 0.0 {
            self.failed = true;
        }
    }
}

fn matrix_is_finite(m: &Matrix) -> bool {
    m.x_axis.is_finite() && m.y_axis.is_finite()
}

pub fn update_particles_health(particles: &mut [Particle]) {
    for particle in particles.iter_mut() {
        particle.update_health();
    }
}
