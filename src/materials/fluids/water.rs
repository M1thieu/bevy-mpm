//! Water fluid material
//!
//! Handles water pressure, viscosity, and EOS calculations.

use crate::config::{EOS_POWER, EOS_STIFFNESS, REST_DENSITY};
use crate::core::Particle;
use crate::materials::utils;
use bevy::prelude::*;

/// Calculate water stress (original logic from P2G)
pub fn calculate_stress(
    particle: &Particle,
    density: f32,
    volume_correction_strength: f32,
    preserve_volume: bool,
    dynamic_viscosity: f32,
) -> Mat2 {
    let volume = particle.mass * utils::inv_exact(density);

    // Original EOS pressure
    let eos_pressure = f32::max(
        -0.1,
        EOS_STIFFNESS * ((density / REST_DENSITY).powi(EOS_POWER as i32) - 1.0),
    );

    // Volume preservation correction (if enabled)
    let volume_correction = if preserve_volume {
        let current_volume = volume;
        let target_volume = particle.volume0;
        let volume_deviation = (current_volume - target_volume) / target_volume;
        volume_correction_strength * volume_deviation * REST_DENSITY
    } else {
        0.0
    };

    // Combined pressure
    let total_pressure = eos_pressure + volume_correction;
    let stress = Mat2::IDENTITY * -total_pressure;

    // Viscosity: deviatoric strain rate for incompressible flow
    let strain_rate = (particle.velocity_gradient + particle.velocity_gradient.transpose()) * 0.5;
    let trace = strain_rate.col(0).x + strain_rate.col(1).y;
    let deviatoric_strain = strain_rate - Mat2::from_diagonal(Vec2::splat(trace * 0.5));
    let viscosity_term = 2.0 * dynamic_viscosity * deviatoric_strain;

    stress + viscosity_term
}
