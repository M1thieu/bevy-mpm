//! Material particles for MPM simulation
//!
//! Particles carry position, velocity, mass and material properties.

use crate::materials::MaterialType;
use crate::math::{
    Matrix, Real, Vector, identity_matrix, matrix_determinant, matrix_trace, zero_matrix,
    zero_vector,
};

/// Boundary contact information stored alongside a particle when interaction
/// with static geometry is enabled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleContact {
    pub boundary_normal: Vector,
    pub boundary_distance: Real,
}

impl Default for ParticleContact {
    fn default() -> Self {
        Self {
            boundary_normal: zero_vector(),
            boundary_distance: 0.0,
        }
    }
}

/// Fracture-related parameters used by snow / brittle materials.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleFracture {
    pub crack_propagation_factor: Real,
    pub crack_threshold: Real,
}

impl Default for ParticleFracture {
    fn default() -> Self {
        Self {
            crack_propagation_factor: 0.0,
            crack_threshold: Real::MAX,
        }
    }
}

/// Internal material state carried per particle for plasticity / hardening.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticlePlasticityState {
    pub nacc_alpha: Real,
    pub plastic_hardening: Real,
    pub elastic_hardening: Real,
    pub log_volume_gain: Real,
}

impl Default for ParticlePlasticityState {
    fn default() -> Self {
        Self {
            nacc_alpha: -0.01,
            plastic_hardening: 1.0,
            elastic_hardening: 1.0,
            log_volume_gain: 0.0,
        }
    }
}

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
    pub plastic_deformation_gradient_det: Real,
    pub material_type: MaterialType,

    // Simulation bookkeeping
    pub grid_index: u64,
    pub phase: Real,
    pub psi_pos: Real,
    pub parameter1: Real,
    pub parameter2: Real,
    pub crack_propagation_factor: Real,
    pub crack_threshold: Real,
    pub cohesion_mass: Real,
    pub cohesion_energy: Real,
    pub phase_buffer: Vector,
    pub is_static: bool,
    pub kinematic_velocity: Option<Vector>,

    // Health tracking
    pub failed: bool,
    pub condition_number: Real,

    // Optional physics extensions
    pub plasticity: ParticlePlasticityState,
    pub contact: Option<ParticleContact>,
    pub fracture: Option<ParticleFracture>,
    pub user_data: u64, // TODO: assess whether these debug slots are useful for LP runtime tooling
    pub debug_value: Real, // TODO: assess whether these debug slots are useful for LP runtime tooling
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
            plastic_deformation_gradient_det: 1.0,
            material_type,
            grid_index: 0,
            phase: 1.0,
            psi_pos: 0.0,
            parameter1: 0.0,
            parameter2: 0.0,
            crack_propagation_factor: 0.0,
            crack_threshold: Real::MAX,
            cohesion_mass: Real::MAX,
            cohesion_energy: 0.0,
            phase_buffer: zero_vector(),
            is_static: false,
            kinematic_velocity: None,
            failed: false,
            condition_number: 1.0,
            plasticity: ParticlePlasticityState::default(),
            contact: None,
            fracture: None,
            user_data: 0,
            debug_value: 0.0,
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

    pub fn with_plasticity(mut self, plasticity: ParticlePlasticityState) -> Self {
        self.plasticity = plasticity;
        self
    }

    pub fn with_contact(mut self, contact: ParticleContact) -> Self {
        self.contact = Some(contact);
        self
    }

    pub fn clear_contact(&mut self) {
        self.contact = None;
    }

    pub fn with_fracture(mut self, fracture: ParticleFracture) -> Self {
        self.fracture = Some(fracture);
        self
    }

    pub fn clear_fracture(&mut self) {
        self.fracture = None;
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
    pub fn plastic_jacobian(&self) -> Real {
        self.plastic_deformation_gradient_det
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
