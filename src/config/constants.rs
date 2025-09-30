// Physical constants for MPM simulation
use bevy::prelude::*;

// Global physics
pub const GRAVITY: Vec2 = Vec2::new(0.0, -80.0);

// Fluid material constants
pub const REST_DENSITY: f32 = 2.0;

// Equation of state parameters
pub const EOS_STIFFNESS: f32 = 2.5;
pub const EOS_POWER: u8 = 4;
