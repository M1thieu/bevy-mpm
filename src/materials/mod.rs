//! Materials for MPM simulation
//!
//! Three categories:
//!
//! * `fluid` - Water and other fluids
//! * `solid` - Elastic materials (coming soon)  
//! * `granular` - Sand-like materials (coming soon)

pub mod fluids;
pub mod granular;
pub mod material_types;
pub mod solids;
pub mod utils;

// Re-export the main material type for convenience
pub use material_types::{MaterialModel, MaterialType};

// Re-export physics utilities for easy access
pub use utils::check;
pub use utils::physics;
