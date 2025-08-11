//! Materials for MPM simulation
//! 
//! Three categories:
//! 
//! * `fluid` - Water and other fluids
//! * `solid` - Elastic materials (coming soon)  
//! * `granular` - Sand-like materials (coming soon)

pub mod material_types;
pub mod fluid;
pub mod solid;  
pub mod granular;
pub mod utils;

// Re-export the main material type for convenience
pub use material_types::MaterialType;

/// Basic properties that all materials have
#[derive(Debug, Clone, Copy)]
pub struct MaterialProperties {
    pub density: f32,
    pub name: &'static str,
    pub is_fluid: bool,
    pub incompressible: bool,
}

impl MaterialProperties {
    pub const fn fluid(name: &'static str, density: f32) -> Self {
        Self {
            density,
            name,
            is_fluid: true,
            incompressible: true,
        }
    }
    
    pub const fn solid(name: &'static str, density: f32) -> Self {
        Self {
            density,
            name,
            is_fluid: false,
            incompressible: false,
        }
    }
    
    pub const fn granular(name: &'static str, density: f32) -> Self {
        Self {
            density,
            name,
            is_fluid: false,
            incompressible: false,
        }
    }
}