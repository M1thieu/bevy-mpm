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

// New helper function to calculate grid weights and positions
pub fn calculate_grid_weights(particle_position: Vec2) -> (UVec2, [Vec2; 3]) {
    let cell_index = particle_position.as_uvec2();
    let cell_difference = (particle_position - cell_index.as_vec2()) - 0.5;

    let weights = [
        0.5 * (0.5 - cell_difference).powf(2.0),
        0.75 - cell_difference.powf(2.0),
        0.5 * (0.5 + cell_difference).powf(2.0),
    ];

    (cell_index, weights)
}

pub fn zero_grid(mut grid: ResMut<Grid>) {
    grid.cells.iter_mut().for_each(|cell| cell.zero());
}

pub fn calculate_grid_velocities(
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