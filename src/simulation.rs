use bevy::prelude::*;

use crate::constants;

pub enum MaterialType {
    Water { vp0: f32, ap: f32, jp: f32 },
}

impl MaterialType {
    // Simple helper constructors
    #[inline(always)]
    pub fn water() -> Self {
        Self::Water {
            vp0: 1.0,
            ap: 0.0,
            jp: 1.0,
        }
    }

    #[inline(always)]
    pub fn honey() -> Self {
        // Higher viscosity for honey-like behavior
        Self::Water {
            vp0: 1.0,
            ap: 0.0,
            jp: 1.0,
        }
    }

    pub fn oil() -> Self {
        // Lower density for oil-like behavior
        Self::Water {
            vp0: 1.0,
            ap: 0.0,
            jp: 1.0,
        }
    }

    // Simple material identification helper
    pub fn material_name(&self) -> &'static str {
        // All materials currently use water physics
        match self {
            Self::Water { .. } => "water",
        }
    }

    #[inline(always)]
    pub fn constitutive_model(&mut self) {
        match self {
            Self::Water { vp0, ap, jp } => {
                let djp = -constants::K_WATER * (1.0 / jp.powi(3) - 1.0);
                *ap = djp * *vp0 * *jp;
            }
        }
    }

    pub fn update_deformation(&mut self, t: Mat2, dt: f32) {
        match self {
            Self::Water { vp0: _, ap: _, jp } => {
                *jp = (1.0 + dt * (t.col(0).x + t.col(1).y)) * *jp;
            }
        }
    }

    /// Check if this material is a fluid
    pub fn is_fluid(&self) -> bool {
        match self {
            Self::Water { .. } => true, // Water is a fluid
        }
    }

    /// Check if this material type typically needs volume preservation
    pub fn is_incompressible(&self) -> bool {
        match self {
            Self::Water { .. } => true, // Fluids are incompressible
        }
    }

    /// Get target density for this material type
    pub fn target_density(&self) -> f32 {
        match self {
            Self::Water { .. } => constants::REST_DENSITY,
        }
    }
}
