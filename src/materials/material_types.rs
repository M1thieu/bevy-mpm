//! Material types for simulation

use crate::materials;
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub enum MaterialType {
    Water { vp0: f32, ap: f32, jp: f32 },
}

impl MaterialType {
    pub fn water() -> Self {
        Self::Water {
            vp0: 1.0,
            ap: 0.0,
            jp: 1.0,
        }
    }

    pub fn is_fluid(&self) -> bool {
        match self {
            Self::Water { .. } => true,
        }
    }

    pub fn material_name(&self) -> &'static str {
        match self {
            Self::Water { .. } => "water",
        }
    }

    pub fn constitutive_model(&mut self) {
        match self {
            Self::Water { vp0, ap, jp } => {
                *ap = materials::fluid::water::apply_constitutive_model(*vp0, *jp);
            }
        }
    }

    pub fn update_deformation(&mut self, t: Mat2, dt: f32) {
        match self {
            Self::Water { vp0: _, ap: _, jp } => {
                materials::fluid::water::update_deformation(jp, t, dt);
            }
        }
    }
}
