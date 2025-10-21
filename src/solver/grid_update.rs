use bevy::prelude::*;

use crate::core::MpmState;

/// Grid update stage (divides momentum by mass, applies gravity, clamps boundaries).
pub fn grid_update(time: Res<Time>, mut state: ResMut<MpmState>) {
    let dt = time.delta_secs();
    state.integrate_grid_velocities(dt);
}
