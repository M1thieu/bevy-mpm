//! Helper functions for materials
//!
//! Math and utility functions that different materials can use.

use bevy::prelude::*;

/// Exact zero check inverse (prevents NaN from division by zero)
#[inline(always)]
pub fn inv_exact(e: f32) -> f32 {
    if e == 0.0 { 0.0 } else { 1.0 / e }
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

/// Physics parameter conversions - universal MPM utilities
/// Used by all professional MPM implementations for material calculations
pub mod physics {
    use bevy::prelude::*;

    /// Computes the Lamé parameters (lambda, mu) from Young's modulus and Poisson ratio
    /// These are fundamental for any solid material stress calculations
    #[inline]
    pub fn lame_lambda_mu(young_modulus: f32, poisson_ratio: f32) -> (f32, f32) {
        let lambda =
            young_modulus * poisson_ratio / ((1.0 + poisson_ratio) * (1.0 - 2.0 * poisson_ratio));
        let mu = shear_modulus(young_modulus, poisson_ratio);
        (lambda, mu)
    }

    /// Shear modulus (mu) from Young's modulus and Poisson ratio
    #[inline]
    pub fn shear_modulus(young_modulus: f32, poisson_ratio: f32) -> f32 {
        young_modulus / (2.0 * (1.0 + poisson_ratio))
    }

    /// Bulk modulus from Young's modulus and Poisson ratio
    /// Measures resistance to compression
    #[inline]
    pub fn bulk_modulus(young_modulus: f32, poisson_ratio: f32) -> f32 {
        young_modulus / (3.0 * (1.0 - 2.0 * poisson_ratio))
    }

    /// Shear modulus from Lamé parameters (just returns mu)
    #[inline]
    pub fn shear_modulus_from_lame(_lambda: f32, mu: f32) -> f32 {
        mu
    }

    /// Bulk modulus from Lamé parameters
    #[inline]
    pub fn bulk_modulus_from_lame(lambda: f32, mu: f32) -> f32 {
        lambda + 2.0 * mu / 3.0
    }


    /// Extracts the strain rate (symmetric) part of velocity gradient
    #[inline]
    pub fn strain_rate(velocity_gradient: &Mat2) -> Mat2 {
        (*velocity_gradient + velocity_gradient.transpose()) * 0.5
    }

    /// Extracts deviatoric part of tensor (removes spherical part)
    #[inline]
    pub fn deviatoric_part(tensor: &Mat2) -> Mat2 {
        let spherical = spherical_part(tensor);
        *tensor - Mat2::from_diagonal(Vec2::splat(spherical))
    }

    /// Extracts spherical part of tensor (mean of diagonal)
    #[inline]
    pub fn spherical_part(tensor: &Mat2) -> f32 {
        (tensor.col(0).x + tensor.col(1).y) * 0.5
    }
}

/// Check if material properties make sense
pub mod check {
    #[inline]
    pub fn density_ok(density: f32) -> bool {
        density > 0.0 && density < 50000.0 && density.is_finite()
    }

    #[inline]
    pub fn viscosity_ok(viscosity: f32) -> bool {
        viscosity >= 0.0 && viscosity < 1e6 && viscosity.is_finite()
    }

    /// Check if deformation gradient determinant is reasonable
    #[inline]
    pub fn deformation_gradient_ok(det: f32) -> bool {
        det > 1e-6 && det < 1e6 && det.is_finite()
    }

    /// Check if Young's modulus is physically reasonable
    #[inline]
    pub fn young_modulus_ok(e: f32) -> bool {
        e > 0.0 && e < 1e12 && e.is_finite()
    }

    /// Check if Poisson ratio is in valid range
    #[inline]
    pub fn poisson_ratio_ok(nu: f32) -> bool {
        nu > -1.0 && nu < 0.5 && nu.is_finite()
    }
}
