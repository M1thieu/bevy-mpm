//! Background grid for MPM simulation
//!
//! 128x128 grid with 3x3 B-spline interpolation.

use crate::materials::utils;
use bevy::prelude::*;
use std::collections::HashMap;

/// Grid dimensions (128x128 cells)
pub const GRID_RESOLUTION: usize = 128;
/// Number of neighbors in 3x3 kernel
pub const NEIGHBOR_COUNT: usize = 9;
/// MPM kernel size (3x3 B-spline)
pub const KERNEL_SIZE: usize = 3;

// Native coordinate offsets for 3x3 B-spline kernel
pub const COORD_OFFSETS: [IVec2; NEIGHBOR_COUNT] = [
    IVec2::new(-1, -1),
    IVec2::new(0, -1),
    IVec2::new(1, -1), // Top row
    IVec2::new(-1, 0),
    IVec2::new(0, 0),
    IVec2::new(1, 0), // Middle row
    IVec2::new(-1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 1), // Bottom row
];

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
    cells: HashMap<(i32, i32), Cell>,
    active_bounds: Option<(IVec2, IVec2)>, // min, max for optimization
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            active_bounds: None,
        }
    }

    /// Get cell at coordinates (read-only)
    pub fn get_cell_coord(&self, coord: IVec2) -> Option<&Cell> {
        self.cells.get(&(coord.x, coord.y))
    }

    /// Get cell at coordinates, creating if needed
    pub fn get_cell_coord_mut(&mut self, coord: IVec2) -> &mut Cell {
        // Update bounds
        if let Some((min, max)) = &mut self.active_bounds {
            min.x = min.x.min(coord.x);
            min.y = min.y.min(coord.y);
            max.x = max.x.max(coord.x);
            max.y = max.y.max(coord.y);
        } else {
            self.active_bounds = Some((coord, coord));
        }

        self.cells
            .entry((coord.x, coord.y))
            .or_insert_with(Cell::zeroed)
    }

    /// Iterator over active cells (coordinates and cell data)
    pub fn iter_active_cells(&self) -> impl Iterator<Item = ((i32, i32), &Cell)> {
        self.cells.iter().map(|(&coords, cell)| (coords, cell))
    }

    /// Mutable iterator over active cells
    pub fn iter_active_cells_mut(&mut self) -> impl Iterator<Item = ((i32, i32), &mut Cell)> {
        self.cells.iter_mut().map(|(&coords, cell)| (coords, cell))
    }

    /// Zero all active cells
    pub fn zero_active_cells(&mut self) {
        for cell in self.cells.values_mut() {
            cell.zero();
        }
    }

    /// Get count of active cells
    pub fn active_cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Clear empty cells (garbage collection)
    pub fn cleanup_empty_cells(&mut self) {
        self.cells.retain(|_, cell| cell.mass > 0.0);

        // Recalculate bounds
        if self.cells.is_empty() {
            self.active_bounds = None;
        } else {
            let mut min = IVec2::new(i32::MAX, i32::MAX);
            let mut max = IVec2::new(i32::MIN, i32::MIN);

            for &(x, y) in self.cells.keys() {
                min.x = min.x.min(x);
                min.y = min.y.min(y);
                max.x = max.x.max(x);
                max.y = max.y.max(y);
            }

            self.active_bounds = Some((min, max));
        }
    }
}

// Fast B-spline weight calculation using optimized math
#[inline(always)]
fn calculate_bspline_weight(d: f32) -> [f32; 3] {
    let d2 = d * d;

    [
        0.5 * (0.5 - d) * (0.5 - d), // Faster than powi(2)
        0.75 - d2,
        0.5 * (0.5 + d) * (0.5 + d),
    ]
}

/// Native coordinate-based grid interpolation - no linear indices anywhere
pub struct GridInterpolation {
    pub base_cell: IVec2,   // Base cell coordinates (i32 for negative bounds)
    pub weights: [Vec2; 3], // B-spline weights [x, y] for 3x3 kernel
    pub neighbor_coords: [IVec2; NEIGHBOR_COUNT], // Direct coordinate access
    pub cell_distances: [Vec2; NEIGHBOR_COUNT], // Distance vectors for APIC
}

impl GridInterpolation {
    /// Native coordinate-based interpolation - no linear indices anywhere
    #[inline(always)]
    pub fn compute_for_particle(particle_position: Vec2) -> Self {
        // Base cell (bottom-left of 3x3 kernel) using i32 for potential negative coords
        let base_cell = IVec2::new(
            particle_position.x.floor() as i32 - 1,
            particle_position.y.floor() as i32 - 1,
        );

        // Cell difference for B-spline weights (relative to base+1)
        let center_cell = base_cell + IVec2::ONE;
        let cell_difference = particle_position - center_cell.as_vec2() - 0.5;

        let x_weights = calculate_bspline_weight(cell_difference.x);
        let y_weights = calculate_bspline_weight(cell_difference.y);

        let weights = [
            Vec2::new(x_weights[0], y_weights[0]),
            Vec2::new(x_weights[1], y_weights[1]),
            Vec2::new(x_weights[2], y_weights[2]),
        ];

        // Generate 3x3 neighbor coordinates directly (no linear indices)
        let mut neighbor_coords = [IVec2::ZERO; NEIGHBOR_COUNT];
        let mut cell_distances = [Vec2::ZERO; NEIGHBOR_COUNT];

        for gy in 0..3 {
            for gx in 0..3 {
                let idx = gy * 3 + gx;
                let coord = base_cell + IVec2::new(gx as i32, gy as i32);
                neighbor_coords[idx] = coord;
                cell_distances[idx] = (coord.as_vec2() - particle_position) + 0.5;
            }
        }

        Self {
            base_cell,
            weights,
            neighbor_coords,
            cell_distances,
        }
    }

    /// Get interpolation weight for a specific neighbor (coordinate-based)
    #[inline(always)]
    pub fn weight_for_neighbor(&self, neighbor_idx: usize) -> f32 {
        let gx = neighbor_idx % KERNEL_SIZE;
        let gy = neighbor_idx / KERNEL_SIZE;
        self.weights[gx].x * self.weights[gy].y
    }

    /// Get coordinate for a specific neighbor (main interface for grid access)
    #[inline(always)]
    pub fn neighbor_coord(&self, neighbor_idx: usize) -> IVec2 {
        self.neighbor_coords[neighbor_idx]
    }

    /// Iterator over (coordinate, weight, distance) tuples for efficient processing
    #[inline(always)]
    pub fn iter_neighbors(&self) -> impl Iterator<Item = (IVec2, f32, Vec2)> + '_ {
        (0..NEIGHBOR_COUNT).map(move |idx| {
            (
                self.neighbor_coords[idx],
                self.weight_for_neighbor(idx),
                self.cell_distances[idx],
            )
        })
    }
}

// Native coordinate-based API - replaces old calculate_grid_weights
#[inline(always)]
pub fn calculate_grid_interpolation(particle_position: Vec2) -> GridInterpolation {
    GridInterpolation::compute_for_particle(particle_position)
}

// Native coordinate-based bounds checking (sparse grid compatible)
#[inline(always)]
pub fn is_valid_grid_coord(coord: IVec2) -> bool {
    coord.x >= 0
        && coord.x < GRID_RESOLUTION as i32
        && coord.y >= 0
        && coord.y < GRID_RESOLUTION as i32
}

// Coordinate-based neighborhood validation (for boundary conditions)
#[inline(always)]
pub fn is_coord_neighborhood_safe(center: IVec2) -> bool {
    // Check if all 3x3 neighbors are within grid bounds
    for dy in -1..=1 {
        for dx in -1..=1 {
            let neighbor = center + IVec2::new(dx, dy);
            if !is_valid_grid_coord(neighbor) {
                return false;
            }
        }
    }
    true
}

#[inline(always)]
pub fn zero_grid(mut grid: ResMut<Grid>) {
    grid.zero_active_cells();
}

#[inline(always)]
pub fn cleanup_grid_cells(mut grid: ResMut<Grid>) {
    grid.cleanup_empty_cells();
}

// Boundary handling modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundaryHandling {
    Stick, // Particles stick to walls (old behavior)
    Slip,  // Particles slide along walls (realistic)
    None,  // No boundary (open world)
}

// Native coordinate-based boundary system (sparse grid compatible)
#[inline(always)]
pub fn calculate_grid_velocities(time: Res<Time>, mut grid: ResMut<Grid>, gravity: Vec2) {
    for (coords, cell) in grid.iter_active_cells_mut() {
        if cell.mass > 0.0 {
            let gravity_velocity = time.delta_secs() * gravity;
            cell.velocity *= utils::inv_exact(cell.mass);
            cell.velocity += gravity_velocity;

            // Native coordinate-based boundary checking (no conversions)
            let coord = IVec2::new(coords.0, coords.1);
            apply_boundary_conditions_coord(cell, coord, BoundaryHandling::Slip);
        }
    }
}

// Native coordinate-based boundary conditions (eliminates conversions)
#[inline(always)]
fn apply_boundary_conditions_coord(cell: &mut Cell, coord: IVec2, boundary_type: BoundaryHandling) {
    // Check if we're near boundaries (coordinate-native)
    let near_boundary = coord.x < 2
        || coord.x > GRID_RESOLUTION as i32 - 3
        || coord.y < 2
        || coord.y > GRID_RESOLUTION as i32 - 3;

    if near_boundary {
        match boundary_type {
            BoundaryHandling::Stick => {
                // Sticky walls
                if coord.x < 2 || coord.x > GRID_RESOLUTION as i32 - 3 {
                    cell.velocity.x = 0.0;
                }
                if coord.y < 2 || coord.y > GRID_RESOLUTION as i32 - 3 {
                    cell.velocity.y = 0.0;
                }
            }
            BoundaryHandling::Slip => {
                // Realistic sliding behavior
                if coord.x < 2 || coord.x > GRID_RESOLUTION as i32 - 3 {
                    cell.velocity.x = 0.0; // Allow Y sliding
                }
                if coord.y < 2 || coord.y > GRID_RESOLUTION as i32 - 3 {
                    cell.velocity.y = 0.0; // Allow X sliding
                }
            }
            BoundaryHandling::None => {
                // Open world - no boundaries (particles flow freely)
            }
        }
    }
}
