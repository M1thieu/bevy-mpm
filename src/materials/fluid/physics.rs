//! Fluid physics calculations
//!
//! Stress, pressure, and deformation for fluid materials.

use crate::materials::utils;
use crate::core::Particle;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct FluidProperties {
    pub density: f32,
    pub viscosity: f32,
}

// Shared fluid physics constants
pub const BULK_MODULUS: f32 = 50.0;
pub const EOS_STIFFNESS: f32 = 10.0;
pub const EOS_POWER: u8 = 4;

pub fn calculate_stress(
    particle: &Particle,
    density: f32,
    properties: FluidProperties,
    volume_correction_strength: f32,
    preserve_volume: bool,
) -> Mat2 {
    let volume = particle.mass * utils::safe_inverse(density);

    // EOS pressure calculation
    let eos_pressure = f32::max(
        -0.1,
        EOS_STIFFNESS * ((density / properties.density).powi(EOS_POWER as i32) - 1.0),
    );

    // Volume preservation correction
    let volume_correction = if preserve_volume {
        let current_volume = volume;
        let target_volume = particle.volume0;
        let volume_deviation = (current_volume - target_volume) / target_volume;
        volume_correction_strength * volume_deviation * properties.density
    } else {
        0.0
    };

    // Combined pressure stress
    let total_pressure = eos_pressure + volume_correction;
    let stress = Mat2::IDENTITY * -total_pressure;

    // Viscosity stress component
    let dudv = particle.affine_momentum_matrix;
    let mut strain = dudv;
    let trace = strain.col(1).x + strain.col(0).y;
    strain.col_mut(0).y = trace;
    strain.col_mut(1).x = trace;
    let viscosity_term = properties.viscosity * strain;

    stress + viscosity_term
}

pub fn apply_constitutive_model(vp0: f32, jp: f32) -> f32 {
    let djp = -BULK_MODULUS * (utils::safe_inverse(jp.powi(3)) - 1.0);
    djp * vp0 * jp
}

pub fn update_deformation(jp: &mut f32, velocity_gradient: Mat2, dt: f32) {
    *jp *= 1.0 + dt * (velocity_gradient.col(0).x + velocity_gradient.col(1).y);
}
