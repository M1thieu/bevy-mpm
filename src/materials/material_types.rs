//! Material types for simulation
//!
//! Keeping the abstraction thin lets the solver stay material-agnostic while
//! still delegating to specialised behaviour for each constitutive model.

use bevy::prelude::*;

use crate::config::SolverParams;
use crate::core::Particle;
use crate::materials::families::FluidParams;
use crate::materials::fluids::water;

use crate::math::Matrix;

/// Shared behaviour that every material must implement.
pub trait MaterialModel {
    fn compute_stress(&self, particle: &Particle, density: f32, params: &SolverParams) -> Matrix;
    fn project_deformation(&self, particle: &mut Particle);
}

#[derive(Component, Debug, Clone)]
pub enum MaterialType {
    Fluid(FluidParams),
}

impl MaterialType {
    pub fn water() -> Self {
        Self::Fluid(FluidParams::water())
    }

    pub fn fluid(params: FluidParams) -> Self {
        Self::Fluid(params)
    }

    pub fn is_fluid(&self) -> bool {
        matches!(self, Self::Fluid(_))
    }

    pub fn material_name(&self) -> &'static str {
        match self {
            Self::Fluid(fluid) => fluid.name,
        }
    }
}

impl MaterialModel for MaterialType {
    fn compute_stress(&self, particle: &Particle, density: f32, params: &SolverParams) -> Matrix {
        match self {
            MaterialType::Fluid(fluid) => water::calculate_stress(particle, density, params, fluid),
        }
    }

    fn project_deformation(&self, particle: &mut Particle) {
        match self {
            MaterialType::Fluid(_) => water::project_deformation(particle),
        }
    }
}
