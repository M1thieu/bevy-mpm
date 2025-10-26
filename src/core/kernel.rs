use bevy::prelude::IVec2;

use crate::math::{Real, Vector};

use super::grid::GridInterpolation;
use super::particle_set::ParticleTransferCache;

/// Compute the inverse dimension factor used by MLS-MPM kernels.
///
/// Sparkl names this `Kernel::inv_d(dx)`. Keeping it in one place helps keep
/// solver code consistent between P2G and G2P.
#[inline]
pub fn inv_d(cell_width: Real) -> Real {
    4.0 / (cell_width * cell_width)
}

/// Convert a particle position into the associated grid cell coordinate.
#[inline]
pub fn cell_from_position(position: Vector, cell_width: Real) -> IVec2 {
    let inv = 1.0 / cell_width;
    IVec2::new(
        (position.x * inv).round() as i32,
        (position.y * inv).round() as i32,
    )
}

/// Populate the cached quadratic B-spline weights and distances for a particle.
#[inline]
pub fn populate_transfer_cache(position: Vector, cache: &mut ParticleTransferCache) {
    let interpolation = GridInterpolation::compute_for_particle(position);
    for (entry, (coord, weight, distance)) in cache
        .neighbors
        .iter_mut()
        .zip(interpolation.iter_neighbors())
    {
        *entry = (coord, weight, distance);
    }
}
