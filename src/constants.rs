// Physical constants for water simulation
use bevy::prelude::*;

// Water properties
pub const RHO_WATER: f32 = 1.0;
pub const K_WATER: f32 = 50.0;
pub const GAMMA_WATER: f32 = 3.0;

pub const GRAVITY: Vec2 = Vec2::new(0.0, -80.0);

// Pressure and fluid constants
pub const EOS_STIFFNESS: f32 = 10.0;
pub const EOS_POWER: u8 = 4;
pub const REST_DENSITY: f32 = 2.0;
pub const DYNAMIC_VISCOSITY: f32 = 0.1;
