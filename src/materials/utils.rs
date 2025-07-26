//! Helper functions for materials
//! 
//! Math and utility functions that different materials can use.

use bevy::prelude::*;

/// Safe division to prevent crashes
#[inline(always)]
pub fn safe_inverse(e: f32) -> f32 {
    if e.abs() < f32::EPSILON { 0.0 } else { 1.0 / e }
}

/// Average pressure being applied
#[inline]
pub fn pressure(stress: Mat2) -> f32 {
    (stress.col(0).x + stress.col(1).y) / 2.0
}

/// How much stress is being applied overall
#[inline]
pub fn stress_magnitude(stress: Mat2) -> f32 {
    (stress.col(0).length_squared() + stress.col(1).length_squared()).sqrt()
}

/// Check if material properties make sense
pub mod check {
    #[inline]
    pub fn density_ok(density: f32) -> bool {
        density > 0.0 && density < 50000.0
    }
    
    #[inline]
    pub fn viscosity_ok(viscosity: f32) -> bool {
        viscosity >= 0.0 && viscosity < 1e6
    }
}