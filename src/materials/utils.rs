//! Helper functions for materials
//!
//! Math and utility functions shared between material models.

use crate::math::{self, Matrix, Real};

/// Exact zero check inverse (prevents NaN from division by zero)
#[inline(always)]
pub fn inv_exact(e: Real) -> Real {
    if e == 0.0 { 0.0 } else { 1.0 / e }
}

/// Average pressure being applied.
#[inline]
pub fn pressure(stress: Matrix) -> Real {
    math::matrix_trace(&stress) / 2.0
}

/// How much stress is being applied overall.
#[inline]
pub fn stress_magnitude(stress: Matrix) -> Real {
    (stress.x_axis.length_squared() + stress.y_axis.length_squared()).sqrt()
}

/// Physics parameter conversions - universal MPM utilities.
/// Used by many constitutive models for material calculations.
pub mod physics {
    use crate::math::{self, Matrix, Real};

    /// Computes the Lamé parameters (lambda, mu) from Young's modulus and Poisson ratio.
    #[inline]
    pub fn lame_lambda_mu(young_modulus: Real, poisson_ratio: Real) -> (Real, Real) {
        let lambda =
            young_modulus * poisson_ratio / ((1.0 + poisson_ratio) * (1.0 - 2.0 * poisson_ratio));
        let mu = shear_modulus(young_modulus, poisson_ratio);
        (lambda, mu)
    }

    /// Shear modulus (mu) from Young's modulus and Poisson ratio.
    #[inline]
    pub fn shear_modulus(young_modulus: Real, poisson_ratio: Real) -> Real {
        young_modulus / (2.0 * (1.0 + poisson_ratio))
    }

    /// Bulk modulus from Young's modulus and Poisson ratio.
    #[inline]
    pub fn bulk_modulus(young_modulus: Real, poisson_ratio: Real) -> Real {
        young_modulus / (3.0 * (1.0 - 2.0 * poisson_ratio))
    }

    /// Shear modulus from Lamé parameters.
    #[inline]
    pub fn shear_modulus_from_lame(_lambda: Real, mu: Real) -> Real {
        mu
    }

    /// Bulk modulus from Lamé parameters.
    #[inline]
    pub fn bulk_modulus_from_lame(lambda: Real, mu: Real) -> Real {
        lambda + 2.0 * mu / 3.0
    }

    /// Extracts the strain rate (symmetric) part of velocity gradient.
    #[inline]
    pub fn strain_rate(velocity_gradient: &Matrix) -> Matrix {
        (*velocity_gradient + math::matrix_transpose(velocity_gradient)) * 0.5
    }

    /// Extracts deviatoric part of tensor (removes spherical part).
    #[inline]
    pub fn deviatoric_part(tensor: &Matrix) -> Matrix {
        let spherical = spherical_part(tensor);
        *tensor - Matrix::from_diagonal(math::repeat_vector(spherical))
    }

    /// Extracts spherical part of tensor (mean of diagonal).
    #[inline]
    pub fn spherical_part(tensor: &Matrix) -> Real {
        math::matrix_trace(tensor) * 0.5
    }
}

/// Check if material properties make sense.
pub mod check {
    use crate::math::Real;

    #[inline]
    pub fn density_ok(density: Real) -> bool {
        density > 0.0 && density < 50000.0 && density.is_finite()
    }

    #[inline]
    pub fn viscosity_ok(viscosity: Real) -> bool {
        viscosity >= 0.0 && viscosity < 1e6 && viscosity.is_finite()
    }

    /// Check if deformation gradient determinant is reasonable.
    #[inline]
    pub fn deformation_gradient_ok(det: Real) -> bool {
        det > 1e-6 && det < 1e6 && det.is_finite()
    }

    /// Check if Young's modulus is physically reasonable.
    #[inline]
    pub fn young_modulus_ok(e: Real) -> bool {
        e > 0.0 && e < 1e12 && e.is_finite()
    }

    /// Check if Poisson ratio is in valid range.
    #[inline]
    pub fn poisson_ratio_ok(nu: Real) -> bool {
        nu > -1.0 && nu < 0.5 && nu.is_finite()
    }
}
