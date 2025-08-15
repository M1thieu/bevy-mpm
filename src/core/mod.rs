//! Core MPM data structures
//!
//! Fundamental data structures for Material Point Method:
//! - Grid: Background Eulerian grid for interpolation
//! - Particle: Lagrangian material points

pub mod grid;
pub mod particle;

pub use grid::*;
pub use particle::*;
