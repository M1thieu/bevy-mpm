use bevy::prelude::*;

pub const GRID_RESOLUTION: usize = 128;

#[derive(Component, Clone)]
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

// Calculate quadratic B-spline weights for MPM interpolation
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

// Boundary handling modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundaryHandling {
    Stick,      // Particles stick to walls (old behavior)
    Slip,       // Particles slide along walls (realistic)
    None,       // No boundary (open world)
}

// Flexible boundary system for open-world compatibility
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

            // Apply configurable boundary handling
            apply_boundary_conditions(cell, index, BoundaryHandling::Slip);
        }
    }
}

// Configurable boundary conditions system
fn apply_boundary_conditions(cell: &mut Cell, index: usize, boundary_type: BoundaryHandling) {
    let y = index / GRID_RESOLUTION;
    let x = index % GRID_RESOLUTION;
    
    // Check if we're near boundaries (configurable for open world)
    let near_boundary = x < 2 || x > GRID_RESOLUTION - 3 || y < 2 || y > GRID_RESOLUTION - 3;
    
    if near_boundary {
        match boundary_type {
            BoundaryHandling::Stick => {
                // Old behavior - sticky walls
                if x < 2 || x > GRID_RESOLUTION - 3 {
                    cell.velocity.x = 0.0;
                }
                if y < 2 || y > GRID_RESOLUTION - 3 {
                    cell.velocity.y = 0.0;
                }
            }
            BoundaryHandling::Slip => {
                // Realistic sliding behavior
                if x < 2 || x > GRID_RESOLUTION - 3 {
                    // Allow sliding along vertical walls (keep Y velocity)
                    cell.velocity.x = 0.0;
                }
                if y < 2 || y > GRID_RESOLUTION - 3 {
                    // Allow sliding along horizontal walls (keep X velocity)  
                    cell.velocity.y = 0.0;
                }
            }
            BoundaryHandling::None => {
                // Open world - no boundaries (particles can flow out)
                // Do nothing - let particles flow freely
            }
        }
    }
}