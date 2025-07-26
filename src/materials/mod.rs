//! Material system for MPM simulation
//! 
//! Materials are organized into three main categories based on how they behave physically.
//! This makes it easy to find what you need and add new materials without breaking existing code.
//!
//! # Categories
//! 
//! * `fluid` - Materials that flow like water, oil, or honey
//! * `solid` - Materials that hold their shape like rubber or metal  
//! * `granular` - Materials like sand that can flow but also pile up
//!
//! Each category has its own folder with all the related materials inside.


pub mod fluid;
pub mod solid;  
pub mod granular;
pub mod utils;

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