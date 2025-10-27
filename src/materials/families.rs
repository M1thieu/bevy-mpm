//! Shared parameter packs for material families.
//!
//! These structs define the vocabulary we use when configuring families of
//! constitutive models. They are intentionally lightweight so gameplay code,
//! asset loaders, or tooling can populate material data without touching the
//! solver.

use crate::config;

/// Parameters describing a generic fluid material.
#[derive(Debug, Clone, Copy)]
pub struct FluidParams {
    pub name: &'static str,
    pub rest_density: f32,
    pub eos_stiffness: f32,
    pub eos_power: u8,
}

impl FluidParams {
    pub const fn new(
        name: &'static str,
        rest_density: f32,
        eos_stiffness: f32,
        eos_power: u8,
    ) -> Self {
        Self {
            name,
            rest_density,
            eos_stiffness,
            eos_power,
        }
    }

    /// Default parameters matching the current fluid demo.
    pub const fn defaults() -> Self {
        Self::new(
            "fluid",
            config::constants::REST_DENSITY,
            config::constants::EOS_STIFFNESS,
            config::constants::EOS_POWER,
        )
    }

    /// Convenience helper retaining the previous "water" label.
    pub const fn water() -> Self {
        Self::new(
            "water",
            config::constants::REST_DENSITY,
            config::constants::EOS_STIFFNESS,
            config::constants::EOS_POWER,
        )
    }
}

impl Default for FluidParams {
    fn default() -> Self {
        Self::defaults()
    }
}
