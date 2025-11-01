use nalgebra::{Matrix2, Vector2};

pub type Real = f32;
pub const DIM: usize = 2;

pub type Vector = Vector2<Real>;
pub type Matrix = Matrix2<Real>;
pub type Point = Vector2<Real>;

#[inline(always)]
pub fn zero_vector() -> Vector {
    Vector::zeros()
}

#[inline(always)]
pub fn repeat_vector(value: Real) -> Vector {
    Vector::new(value, value)
}

#[inline(always)]
pub fn zero_matrix() -> Matrix {
    Matrix::zeros()
}

#[inline(always)]
pub fn identity_matrix() -> Matrix {
    Matrix::identity()
}

#[inline(always)]
pub fn matrix_trace(m: &Matrix) -> Real {
    m.trace()
}

#[inline(always)]
pub fn matrix_transpose(m: &Matrix) -> Matrix {
    m.transpose()
}

#[inline(always)]
pub fn matrix_determinant(m: &Matrix) -> Real {
    m.determinant()
}

#[inline(always)]
pub fn diagonal_from_value(value: Real) -> Matrix {
    Matrix::from_diagonal(&Vector::new(value, value))
}

#[inline(always)]
pub fn diagonal_from_vec(vec: Vector) -> Matrix {
    Matrix::from_diagonal(&vec)
}

#[inline(always)]
pub fn outer_product(a: Vector, b: Vector) -> Matrix {
    a * b.transpose()
}

#[inline(always)]
pub fn quadratic_bspline_weights(offset: Real) -> [Real; 3] {
    let d2 = offset * offset;

    [
        0.5 * (0.5 - offset) * (0.5 - offset),
        0.75 - d2,
        0.5 * (0.5 + offset) * (0.5 + offset),
    ]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DecomposedTensor {
    pub deviatoric_part: Matrix,
    pub spherical_part: Real,
}

impl DecomposedTensor {
    pub fn decompose(tensor: &Matrix) -> Self {
        let spherical_part = matrix_trace(tensor) / (DIM as Real);
        let mut deviatoric_part = *tensor;
        deviatoric_part[(0, 0)] -= spherical_part;
        deviatoric_part[(1, 1)] -= spherical_part;
        Self {
            deviatoric_part,
            spherical_part,
        }
    }

    pub fn zero() -> Self {
        Self {
            deviatoric_part: zero_matrix(),
            spherical_part: 0.0,
        }
    }

    pub fn recompose(&self) -> Matrix {
        let mut result = self.deviatoric_part;
        result[(0, 0)] += self.spherical_part;
        result[(1, 1)] += self.spherical_part;
        result
    }
}

// === Bevy Conversion Helpers ===
// Convert nalgebra types to Bevy types for rendering

#[inline(always)]
pub fn to_bevy_vec2(v: &Vector) -> bevy::prelude::Vec2 {
    bevy::prelude::Vec2::new(v.x, v.y)
}

#[inline(always)]
pub fn from_bevy_vec2(v: bevy::prelude::Vec2) -> Vector {
    Vector::new(v.x, v.y)
}

#[inline(always)]
pub fn to_bevy_mat2(m: &Matrix) -> bevy::prelude::Mat2 {
    bevy::prelude::Mat2::from_cols_array(&[
        m[(0, 0)], m[(1, 0)],
        m[(0, 1)], m[(1, 1)],
    ])
}
