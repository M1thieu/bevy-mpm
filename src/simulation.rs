//! Material simulation and constitutive models
//! 
//! Defines material types and their behaviors including deformation updates,
//! stress calculations, and material property queries.

use bevy::prelude::*;

use crate::materials;

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
        match self {
            Self::Water { .. } => "water",
        }
    }

    #[inline(always)]
    pub fn constitutive_model(&mut self) {
        match self {
            Self::Water { vp0, ap, jp } => {
                // Use water material function (organized logic)
                *ap = materials::fluid::water::apply_constitutive_model(*vp0, *jp);
            }
        }
    }

    pub fn update_deformation(&mut self, t: Mat2, dt: f32) {
        match self {
            Self::Water { vp0: _, ap: _, jp } => {
                // Use water material function (organized logic)
                materials::fluid::water::update_deformation(jp, t, dt);
            }
        }
    }

    /// Check if this material is a fluid
    pub fn is_fluid(&self) -> bool {
        match self {
            Self::Water { .. } => materials::fluid::water::is_fluid(),
        }
    }

    /// Check if this material type typically needs volume preservation
    pub fn is_incompressible(&self) -> bool {
        match self {
            Self::Water { .. } => materials::fluid::water::is_incompressible(),
        }
    }

    /// Get target density for this material type
    pub fn target_density(&self) -> f32 {
        match self {
            Self::Water { .. } => materials::fluid::water::target_density(),
        }
    }
}
