use bevy::prelude::*;

/// Solver parameters for controlling MPM simulation behavior
#[derive(Resource, Clone)]
pub struct SolverParams {
    /// Enable volume preservation for incompressible materials (like water)
    /// When true, applies density correction to maintain volume conservation
    pub preserve_fluid_volume: bool,

    /// Strength of volume preservation correction (0.0 = disabled, 1.0 = strong)
    pub volume_correction_strength: f32,

    /// Dynamic viscosity for fluid materials
    pub dynamic_viscosity: f32,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            preserve_fluid_volume: false, // EOS handles volume naturally
            volume_correction_strength: 0.0,
            dynamic_viscosity: 0.001,
        }
    }
}

impl SolverParams {
    /// Create solver parameters with volume preservation enabled
    pub fn with_volume_preservation() -> Self {
        Self {
            preserve_fluid_volume: true,
            volume_correction_strength: 0.5,
            dynamic_viscosity: 0.001,
        }
    }

    /// Create solver parameters with volume preservation disabled
    pub fn without_volume_preservation() -> Self {
        Self {
            preserve_fluid_volume: false,
            volume_correction_strength: 0.0,
            dynamic_viscosity: 0.001,
        }
    }

    /// Set volume preservation strength (0.0 to 1.0)
    pub fn with_correction_strength(mut self, strength: f32) -> Self {
        self.volume_correction_strength = strength.clamp(0.0, 1.0);
        self
    }
}
