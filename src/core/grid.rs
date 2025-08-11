//! Background grid for MPM simulation
//!
//! 128x128 grid with 3x3 B-spline interpolation.

use bevy::prelude::*;

/// Grid dimensions (128x128 cells)
pub const GRID_RESOLUTION: usize = 128;
/// Number of neighbors in 3x3 kernel
pub const NEIGHBOR_COUNT: usize = 9;
/// MPM kernel size (3x3 B-spline)
pub const KERNEL_SIZE: usize = 3;

// Pre-computed neighbor offsets for 3x3 grid pattern (performance optimization)
pub const NEIGHBOR_OFFSETS: [i32; NEIGHBOR_COUNT] = [
    -(GRID_RESOLUTION as i32) - 1, -(GRID_RESOLUTION as i32), -(GRID_RESOLUTION as i32) + 1, // Top row
    -1, 0, 1,                                                                                  // Middle row  
    (GRID_RESOLUTION as i32) - 1, GRID_RESOLUTION as i32, (GRID_RESOLUTION as i32) + 1,       // Bottom row
];


// Const generic version for compile-time optimization
pub type GridArray<T> = [T; GRID_RESOLUTION * GRID_RESOLUTION];

#[derive(Component, Clone)]
pub struct Cell {
    pub velocity: Vec2,
    pub mass: f32,
}

impl Cell {
    #[inline(always)]
    pub fn zeroed() -> Self {
        Self {
            velocity: Vec2::ZERO,
            mass: 0.0,
        }
    }

    #[inline(always)]
    pub fn zero(&mut self) {
        self.velocity = Vec2::ZERO;
        self.mass = 0.0;
    }
}

#[derive(Resource)]
pub struct Grid {
    pub cells: Vec<Cell>,
}

// Fast B-spline weight calculation using optimized math
#[inline(always)]
fn calculate_bspline_weight(d: f32) -> [f32; 3] {
    let d2 = d * d;
    
    [
        0.5 * (0.5 - d) * (0.5 - d),  // Faster than powi(2)
        0.75 - d2,
        0.5 * (0.5 + d) * (0.5 + d),
    ]
}

// Calculate quadratic B-spline weights for MPM interpolation
#[inline(always)]
pub fn calculate_grid_weights(particle_position: Vec2) -> (UVec2, [Vec2; 3]) {
    let cell_index = particle_position.as_uvec2();
    let cell_difference = (particle_position - cell_index.as_vec2()) - 0.5;

    let x_weights = calculate_bspline_weight(cell_difference.x);
    let y_weights = calculate_bspline_weight(cell_difference.y);

    let weights = [
        Vec2::new(x_weights[0], y_weights[0]),
        Vec2::new(x_weights[1], y_weights[1]),
        Vec2::new(x_weights[2], y_weights[2]),
    ];

    (cell_index, weights)
}


// Bounds checking with early exit
#[inline(always)]
pub fn is_valid_grid_position(pos: UVec2) -> bool {
    pos.x < GRID_RESOLUTION as u32 && pos.y < GRID_RESOLUTION as u32
}

// Fast grid index calculation with bounds check
#[inline(always)]
pub fn safe_grid_index(pos: UVec2) -> Option<usize> {
    if is_valid_grid_position(pos) {
        Some(pos.y as usize * GRID_RESOLUTION + pos.x as usize)
    } else {
        None
    }
}

// Check if entire 3x3 neighborhood around a center index is valid (batch validation)
#[inline(always)]
pub fn is_neighborhood_valid(center_index: usize) -> bool {
    for &offset in NEIGHBOR_OFFSETS.iter() {
        let neighbor_index = center_index as i32 + offset;
        if neighbor_index < 0 || neighbor_index >= (GRID_RESOLUTION * GRID_RESOLUTION) as i32 {
            return false;
        }
        
        // Check for grid edge wrapping
        let center_x = center_index % GRID_RESOLUTION;
        let center_y = center_index / GRID_RESOLUTION;
        let neighbor_x = (neighbor_index as usize) % GRID_RESOLUTION;
        let neighbor_y = (neighbor_index as usize) / GRID_RESOLUTION;
        
        if (center_x as i32 - neighbor_x as i32).abs() > 1 || 
           (center_y as i32 - neighbor_y as i32).abs() > 1 {
            return false;
        }
    }
    true
}

// Get neighbor indices using pre-computed offsets with batch validation
#[inline(always)]
pub fn get_neighbor_indices(center_index: usize) -> [Option<usize>; NEIGHBOR_COUNT] {
    let mut neighbors = [None; NEIGHBOR_COUNT];
    
    // Early exit if entire neighborhood is invalid (batch validation)
    if !is_neighborhood_valid(center_index) {
        return neighbors;
    }
    
    // All neighbors are guaranteed valid, compute directly
    for (i, &offset) in NEIGHBOR_OFFSETS.iter().enumerate() {
        let neighbor_index = (center_index as i32 + offset) as usize;
        neighbors[i] = Some(neighbor_index);
    }
    
    neighbors
}

#[inline(always)]
pub fn zero_grid(mut grid: ResMut<Grid>) {
    grid.cells.iter_mut().for_each(|cell| cell.zero());
}

// Boundary handling modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundaryHandling {
    Stick, // Particles stick to walls (old behavior)
    Slip,  // Particles slide along walls (realistic)
    None,  // No boundary (open world)
}

// Flexible boundary system for open-world compatibility
#[inline(always)]
pub fn calculate_grid_velocities(time: Res<Time>, mut grid: ResMut<Grid>, gravity: Vec2) {
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
#[inline(always)]
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
