use bevy::math::{Mat2, Vec2};

pub type Real = f32;
pub const DIM: usize = 2;

pub type Vector = Vec2;
pub type Matrix = Mat2;
pub type Point = Vec2;

#[inline(always)]
pub fn zero_vector() -> Vector {
    Vec2::ZERO
}

#[inline(always)]
pub fn repeat_vector(value: Real) -> Vector {
    Vec2::splat(value)
}

#[inline(always)]
pub fn zero_matrix() -> Matrix {
    Mat2::ZERO
}

#[inline(always)]
pub fn identity_matrix() -> Matrix {
    Mat2::IDENTITY
}

#[inline(always)]
pub fn matrix_trace(m: &Matrix) -> Real {
    m.x_axis.x + m.y_axis.y
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
    Matrix::from_diagonal(Vec2::splat(value))
}

#[inline(always)]
pub fn diagonal_from_vec(vec: Vector) -> Matrix {
    Matrix::from_diagonal(vec)
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
        deviatoric_part.x_axis.x -= spherical_part;
        deviatoric_part.y_axis.y -= spherical_part;
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
        result.x_axis.x += self.spherical_part;
        result.y_axis.y += self.spherical_part;
        result
    }
}
