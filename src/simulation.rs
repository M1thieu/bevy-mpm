use bevy::prelude::*;

use crate::constants;

pub enum MaterialType {
    Water { vp0: f32, ap: f32, jp: f32 },
}

impl MaterialType {
    // Simple helper constructors
    #[inline(always)]
    pub fn water() -> Self {
        Self::Water { vp0: 1.0, ap: 0.0, jp: 1.0 }
    }
    
    #[inline(always)]
    pub fn honey() -> Self {
        // TODO: Different viscosity parameters
        Self::Water { vp0: 1.0, ap: 0.0, jp: 1.0 }
    }
    
    pub fn oil() -> Self {
        // TODO: Different density/viscosity  
        Self::Water { vp0: 1.0, ap: 0.0, jp: 1.0 }
    }
    
    // Simple material identification helper
    pub fn material_name(&self) -> &'static str {
        // For now all are water physics, but this gives us a hook for the future
        match self {
            Self::Water { .. } => "water", // We could extend this to track preset type
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
}