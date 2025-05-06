use bevy::prelude::*;

use crate::constants;

// Gravity constant moved from the previous implementation
pub const GRAVITY: Vec2 = Vec2::new(0.0, -80.0);

pub enum MaterialType {
    Water { vp0: f32, ap: f32, jp: f32 },
}

impl MaterialType {
    pub fn constitutive_model(&mut self) {
        match self {
            Self::Water { vp0, ap, jp } => {
                let djp = -constants::K_WATER * (1.0 / jp.powf(constants::GAMMA_WATER) - 1.0);
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