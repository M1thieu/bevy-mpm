use bevy::prelude::*;

// === Material Constants ===
pub const RHO_WATER: f32 = 1.0;
pub const K_WATER: f32 = 50.0;
pub const GAMMA_WATER: f32 = 3.0;

// === EOS Constants (moved from p2g.rs) ===
pub const EOS_STIFFNESS: f32 = 10.0;
pub const EOS_POWER: u8 = 4;
pub const REST_DENSITY: f32 = 2.0;
pub const DYNAMIC_VISCOSITY: f32 = 0.1;

// === Physics Constants ===
pub const GRAVITY: Vec2 = Vec2::new(0.0, -80.0);

// === Dispatch Sizes (for shader compatibility) ===
pub const PARTICLE_DISPATCH_SIZE: u32 = 64;
pub const GRID_DISPATCH_SIZE: u32 = 8;
pub const BUKKIT_SIZE: u32 = 6;
pub const BUKKIT_HALO_SIZE: u32 = 1;
