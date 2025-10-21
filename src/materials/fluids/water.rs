//! Water fluid material
//!
//! Handles water pressure, viscosity, and EOS calculations.

use crate::config::{EOS_POWER, EOS_STIFFNESS, REST_DENSITY, SolverParams};
use crate::core::Particle;
use crate::materials::utils;
use crate::math;
use crate::math::{Matrix, Real};

/// Calculate water stress (original logic from P2G)
pub fn calculate_stress(particle: &Particle, density: Real, params: &SolverParams) -> Matrix {
    let volume = particle.mass * utils::inv_exact(density);

    // Original EOS pressure
    let eos_pressure = Real::max(
        -0.1,
        EOS_STIFFNESS * ((density / REST_DENSITY).powi(EOS_POWER as i32) - 1.0),
    );

    // Volume preservation correction (if enabled)
    let volume_correction = if params.preserve_fluid_volume {
        let current_volume = volume;
        let target_volume = particle.volume0;
        let volume_deviation = (current_volume - target_volume) / target_volume;
        params.volume_correction_strength * volume_deviation * REST_DENSITY
    } else {
        0.0
    };

    // Combined pressure
    let total_pressure = eos_pressure + volume_correction;
    let stress = math::identity_matrix() * -total_pressure;

    // Viscosity: deviatoric strain rate for incompressible flow
    let strain_rate =
        (particle.velocity_gradient + math::matrix_transpose(&particle.velocity_gradient)) * 0.5;
    let trace = math::matrix_trace(&strain_rate);
    let deviatoric_strain = strain_rate - Matrix::from_diagonal(math::repeat_vector(trace * 0.5));
    let viscosity_term = 2.0 * params.dynamic_viscosity * deviatoric_strain;

    stress + viscosity_term
}

/// Fluids project deformation back to isotropic volume after integration.
pub fn project_deformation(particle: &mut Particle) {
    let jacobian = math::matrix_determinant(&particle.deformation_gradient);
    let scale = jacobian.abs().powf(0.25);
    particle.deformation_gradient = math::identity_matrix() * scale;
}
