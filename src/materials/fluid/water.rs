//! Water fluid material
//!
//! Handles water pressure, viscosity, and EOS calculations.

use crate::config::{EOS_POWER, EOS_STIFFNESS, K_WATER, REST_DENSITY};
use crate::core::Particle;
use crate::materials::utils;
use bevy::prelude::*;

/// Calculate constitutive model for water (original logic from simulation.rs)
pub fn apply_constitutive_model(vp0: f32, jp: f32) -> f32 {
    let djp = -K_WATER * (utils::inv_exact(jp.powi(3)) - 1.0);
    djp * vp0 * jp
}

/// Update water deformation (original logic from simulation.rs)
pub fn update_deformation(jp: &mut f32, velocity_gradient: Mat2, dt: f32) {
    *jp = (1.0 + dt * (velocity_gradient.col(0).x + velocity_gradient.col(1).y)) * *jp;
}

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

    // Add viscosity term (original logic)
    let dudv = particle.affine_momentum_matrix;
    let mut strain = dudv;
    let trace = strain.col(1).x + strain.col(0).y;
    strain.col_mut(0).y = trace;
    strain.col_mut(1).x = trace;
    let viscosity_term = dynamic_viscosity * strain;

    stress + viscosity_term
}

pub mod types {
    use crate::config::REST_DENSITY;
    use crate::materials::MaterialProperties;

    pub const WATER: MaterialProperties = MaterialProperties::fluid("water", REST_DENSITY);
}

/// Check if water material properties
pub fn is_fluid() -> bool {
    true
}
pub fn is_incompressible() -> bool {
    true
}
pub fn target_density() -> f32 {
    REST_DENSITY
}
