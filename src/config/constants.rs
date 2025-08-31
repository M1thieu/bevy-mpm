// Physical constants for MPM simulation
use bevy::prelude::*;

// Global physics
pub const GRAVITY: Vec2 = Vec2::new(0.0, -80.0);

// Fluid material constants
pub const REST_DENSITY: f32 = 2.0;
pub const DYNAMIC_VISCOSITY: f32 = 0.1;

// Water-specific constants
pub const K_WATER: f32 = 50.0;

// Equation of state parameters
pub const EOS_STIFFNESS: f32 = 10.0;
pub const EOS_POWER: u8 = 4;
