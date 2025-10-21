//! Sparse grid wrapper for MLS-MPM simulation.
//!
//! This reworks the legacy dense HashMap grid into a structure backed by
//! `SpGrid<GridNode>`, exposing the same helper APIs (interpolation,
//! neighborhood iteration) so the existing solver code keeps compiling while
//! we progressively port the remaining logic from Sparkl.

use bevy::prelude::*;

use crate::geometry::sp_grid::{pack_from_ivec, unpack_coords, PackedCell, SpGrid};
use crate::math::{zero_vector, Real, Vector};

#[derive(Clone, Debug)]
pub struct GridNode {
    pub mass: Real,
    pub momentum: Vector,
    pub velocity: Vector,
    pub psi_momentum: Real,
    pub psi_mass: Real,
    pub particles: (u32, u32),
    pub active: bool,
    pub boundary: bool,
}

impl Default for GridNode {
    fn default() -> Self {
        Self {
            mass: 0.0,
            momentum: zero_vector(),
            velocity: zero_vector(),
            psi_momentum: 0.0,
            psi_mass: 0.0,
            particles: (0, 0),
            active: false,
            boundary: false,
        }
    }
}

impl GridNode {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn boundary(&self) -> bool {
        self.boundary
    }

    pub fn set_boundary(&mut self, boundary: bool) {
        self.boundary = boundary;
    }
}

/// Grid dimensions (128x128 cells for the current demo).
pub const GRID_RESOLUTION: usize = 128;
/// Number of neighbors in the quadratic (3x3) kernel.
pub const NEIGHBOR_COUNT: usize = 9;
/// Side length of the quadratic kernel.
pub const KERNEL_SIZE: usize = 3;

/// Native coordinate offsets for the 3x3 quadratic B-spline kernel.
pub const COORD_OFFSETS: [IVec2; NEIGHBOR_COUNT] = [
    IVec2::new(-1, -1),
    IVec2::new(0, -1),
    IVec2::new(1, -1),
    IVec2::new(-1, 0),
    IVec2::new(0, 0),
    IVec2::new(1, 0),
    IVec2::new(-1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 1),
];

/// Sparse grid resource storing all active nodes.
#[derive(Resource)]
pub struct Grid {
    cell_width: Real,
    nodes: SpGrid<GridNode>,
}

impl Grid {
    pub fn new() -> Self {
        Self::with_cell_width(1.0)
    }

    pub fn with_cell_width(cell_width: Real) -> Self {
        Self {
            cell_width,
            nodes: SpGrid::new(cell_width),
        }
    }

    pub fn cell_width(&self) -> Real {
        self.cell_width
    }

    #[inline]
    fn packed_id(coord: IVec2) -> PackedCell {
        pack_from_ivec(coord)
    }

    /// Returns a read-only handle to the node at `coord`, if it exists.
    pub fn get_cell_coord(&self, coord: IVec2) -> Option<&GridNode> {
        self.nodes.get_packed(Self::packed_id(coord))
    }

    /// Returns a mutable handle to the node at `coord`, allocating it if needed.
    pub fn get_cell_coord_mut(&mut self, coord: IVec2) -> &mut GridNode {
        let id = Self::packed_id(coord);
        let node = self.nodes.get_packed_mut(id);
        node.set_active(true);
        node
    }

    pub fn iter_active_cells(&self) -> impl Iterator<Item = ((i32, i32), &GridNode)> {
        self.nodes
            .iter_cells()
            .map(|(id, node)| (unpack_coords(id), node))
    }

    pub fn iter_active_cells_mut(
        &mut self,
    ) -> impl Iterator<Item = ((i32, i32), &mut GridNode)> {
        self.nodes
            .iter_cells_mut()
            .map(|(id, node)| (unpack_coords(id), node))
    }

    /// Resets every active node back to the default (zero mass/momentum).
    pub fn zero_active_cells(&mut self) {
        for (_, node) in self.nodes.iter_cells_mut() {
            node.reset();
        }
    }

    /// Reclaims nodes whose mass dropped to zero.
    pub fn cleanup_empty_cells(&mut self) {
        self.nodes.retain(|_, node| {
            let keep = node.mass > 0.0;
            node.set_active(false);
            keep
        });
    }

    pub fn active_cell_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

/// Dense interpolation structure - unchanged API so the old solver keeps working.
#[derive(Clone, Copy)]
pub struct GridInterpolation {
    pub base_cell: IVec2,
    pub weights: [Vec2; KERNEL_SIZE],
    pub neighbor_coords: [IVec2; NEIGHBOR_COUNT],
    pub cell_distances: [Vec2; NEIGHBOR_COUNT],
}

impl GridInterpolation {
    #[inline(always)]
    pub fn compute_for_particle(position: Vec2) -> Self {
        let base_cell = IVec2::new(
            position.x.floor() as i32 - 1,
            position.y.floor() as i32 - 1,
        );

        let center_cell = base_cell + IVec2::ONE;
        let cell_difference = position - center_cell.as_vec2() - 0.5;

        let x_weights = calculate_bspline_weight(cell_difference.x);
        let y_weights = calculate_bspline_weight(cell_difference.y);

        let weights = [
            Vec2::new(x_weights[0], y_weights[0]),
            Vec2::new(x_weights[1], y_weights[1]),
            Vec2::new(x_weights[2], y_weights[2]),
        ];

        let mut neighbor_coords = [IVec2::ZERO; NEIGHBOR_COUNT];
        let mut cell_distances = [Vec2::ZERO; NEIGHBOR_COUNT];

        for gy in 0..3 {
            for gx in 0..3 {
                let idx = gy * 3 + gx;
                let coord = base_cell + IVec2::new(gx as i32, gy as i32);
                neighbor_coords[idx] = coord;
                cell_distances[idx] = (coord.as_vec2() - position) + 0.5;
            }
        }

        Self {
            base_cell,
            weights,
            neighbor_coords,
            cell_distances,
        }
    }

    #[inline(always)]
    pub fn weight_for_neighbor(&self, neighbor_idx: usize) -> f32 {
        let gx = neighbor_idx % KERNEL_SIZE;
        let gy = neighbor_idx / KERNEL_SIZE;
        self.weights[gx].x * self.weights[gy].y
    }

    #[inline(always)]
    pub fn neighbor_coord(&self, neighbor_idx: usize) -> IVec2 {
        self.neighbor_coords[neighbor_idx]
    }

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

fn calculate_bspline_weight(d: f32) -> [f32; 3] {
    let d2 = d * d;

    [
        0.5 * (0.5 - d) * (0.5 - d),
        0.75 - d2,
        0.5 * (0.5 + d) * (0.5 + d),
    ]
}

#[inline(always)]
pub fn calculate_grid_interpolation(particle_position: Vec2) -> GridInterpolation {
    GridInterpolation::compute_for_particle(particle_position)
}

#[inline(always)]
pub fn is_valid_grid_coord(coord: IVec2) -> bool {
    coord.x >= 0
        && coord.x < GRID_RESOLUTION as i32
        && coord.y >= 0
        && coord.y < GRID_RESOLUTION as i32
}

#[inline(always)]
pub fn is_coord_neighborhood_safe(center: IVec2) -> bool {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundaryHandling {
    Stick,
    Slip,
    None,
}

pub fn apply_boundary_conditions(
    node: &mut GridNode,
    coord: IVec2,
    boundary_type: BoundaryHandling,
) {
    let near_boundary = coord.x < 2
        || coord.x > GRID_RESOLUTION as i32 - 3
        || coord.y < 2
        || coord.y > GRID_RESOLUTION as i32 - 3;

    if !near_boundary {
        return;
    }

    match boundary_type {
        BoundaryHandling::Stick => {
            if coord.x < 2 || coord.x > GRID_RESOLUTION as i32 - 3 {
                node.velocity.x = 0.0;
            }
            if coord.y < 2 || coord.y > GRID_RESOLUTION as i32 - 3 {
                node.velocity.y = 0.0;
            }
        }
        BoundaryHandling::Slip => {
            if coord.x < 2 || coord.x > GRID_RESOLUTION as i32 - 3 {
                node.velocity.x = 0.0;
            }
            if coord.y < 2 || coord.y > GRID_RESOLUTION as i32 - 3 {
                node.velocity.y = 0.0;
            }
        }
        BoundaryHandling::None => {}
    }
}
