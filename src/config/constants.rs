// Physical constants for MPM simulation
use crate::math::Vector;

// Global physics
pub const GRAVITY: Vector = Vector::new(0.0, -80.0);

// Fluid material constants
pub const REST_DENSITY: f32 = 2.0;

// Equation of state parameters
pub const EOS_STIFFNESS: f32 = 2.0;
pub const EOS_POWER: u8 = 4;
