//! Material types for simulation
//!
//! Keeping the abstraction thin lets the solver stay material-agnostic while
//! still delegating to specialised behaviour for each constitutive model.

use bevy::prelude::*;

use crate::config::SolverParams;
use crate::core::Particle;
use crate::materials::fluids::water;

/// Shared behaviour that every material must implement.
pub trait MaterialModel {
    fn compute_stress(&self, particle: &Particle, density: f32, params: &SolverParams) -> Mat2;
    fn project_deformation(&self, particle: &mut Particle);
}

#[derive(Component, Debug, Clone)]
pub enum MaterialType {
    Water,
}

impl MaterialType {
    pub fn water() -> Self {
        Self::Water
    }

    pub fn is_fluid(&self) -> bool {
        matches!(self, Self::Water)
    }

    pub fn material_name(&self) -> &'static str {
        match self {
            Self::Water => "water",
        }
    }
}

impl MaterialModel for MaterialType {
    fn compute_stress(&self, particle: &Particle, density: f32, params: &SolverParams) -> Mat2 {
        match self {
            MaterialType::Water => water::calculate_stress(particle, density, params),
        }
    }

    fn project_deformation(&self, particle: &mut Particle) {
        match self {
            MaterialType::Water => water::project_deformation(particle),
        }
    }
}
