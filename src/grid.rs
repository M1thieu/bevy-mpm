use bevy::prelude::*;

// Constants moved from simulation.rs that are related to the grid
pub const GRID_RESOLUTION: usize = 128;

#[derive(Component)]
#[repr(C)] // GPU memory alignment for future WGPU transition
pub struct Cell {
    pub velocity: Vec2,
    pub mass: f32,
}

impl Cell {
    pub fn zeroed() -> Self {
        Self {
            velocity: Vec2::ZERO,
            mass: 0.0,
        }
    }

    pub fn zero(&mut self) {
        self.velocity = Vec2::ZERO;
        self.mass = 0.0;
    }
}

#[derive(Resource)]
pub struct Grid {
    pub cells: Vec<Cell>,
}

// Grid helper functions - calculate weights and positions
pub fn grid_calculate_weights(particle_position: Vec2) -> (UVec2, [Vec2; 3]) {
    let cell_index = particle_position.as_uvec2();
    let cell_difference = (particle_position - cell_index.as_vec2()) - 0.5;

    let weights = [
        0.5 * (0.5 - cell_difference).powf(2.0),
        0.75 - cell_difference.powf(2.0),
        0.5 * (0.5 + cell_difference).powf(2.0),
    ];

    (cell_index, weights)
}

// Weight iteration helper - following EA's quadratic weights pattern
pub fn grid_iter_quadratic_weights(weights: &[Vec2; 3]) -> impl Iterator<Item = (usize, usize, f32)> + '_ {
    (0..3).flat_map(move |gx| {
        (0..3).map(move |gy| (gx, gy, weights[gx].x * weights[gy].y))
    })
}

// Grid cell access with index return (mutable)
pub fn grid_get_cell_mut(grid: &mut Grid, pos: UVec2) -> Option<(usize, &mut Cell)> {
    let idx = pos.y as usize * GRID_RESOLUTION + pos.x as usize;
    grid.cells.get_mut(idx).map(|cell| (idx, cell))
}

// Grid cell access (read-only)
pub fn grid_get_cell(grid: &Grid, pos: UVec2) -> Option<(usize, &Cell)> {
    let idx = pos.y as usize * GRID_RESOLUTION + pos.x as usize;
    grid.cells.get(idx).map(|cell| (idx, cell))
}

pub fn grid_zero(mut grid: ResMut<Grid>) {
    grid.cells.iter_mut().for_each(|cell| cell.zero());
}

pub fn grid_calculate_velocities(
    time: Res<Time>,
    mut grid: ResMut<Grid>,
    gravity: Vec2,
) {
    for (index, cell) in grid.cells.iter_mut().enumerate() {
        if cell.mass > 0.0 {
            let gravity_velocity = time.delta_secs() * gravity;
            cell.velocity /= cell.mass;
            cell.velocity += gravity_velocity;

            // Fixed indexing: index = y * width + x
            // So y = index / width, x = index % width
            let y = index / GRID_RESOLUTION;
            let x = index % GRID_RESOLUTION;

            if x < 2 || x > GRID_RESOLUTION - 3 {
                cell.velocity.x = 0.0;
            }

            if y < 2 || y > GRID_RESOLUTION - 3 {
                cell.velocity.y = 0.0;
            }
        }
    }
}