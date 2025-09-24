//! Material types for simulation

use bevy::prelude::*;

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
