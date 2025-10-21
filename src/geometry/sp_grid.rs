use std::collections::HashMap;

use bevy::prelude::IVec2;

use crate::math::{Real, Vector};

pub type PackedCell = u64;

const NEIGHBOR_OFFSETS: [(i32, i32); 9] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[inline]
pub fn pack_coords(ix: i32, iy: i32) -> PackedCell {
    ((ix as u64) << 32) | (iy as u32 as u64)
}

#[inline]
pub fn pack_from_ivec(coord: IVec2) -> PackedCell {
    pack_coords(coord.x, coord.y)
}

#[inline]
pub fn unpack_coords(id: PackedCell) -> (i32, i32) {
    let ix = (id >> 32) as i32;
    let iy = id as u32 as i32;
    (ix, iy)
}

#[inline]
pub fn unpack_to_ivec(id: PackedCell) -> IVec2 {
    let (ix, iy) = unpack_coords(id);
    IVec2::new(ix, iy)
}

#[derive(Clone)]
pub struct SpGrid<T> {
    cell_width: Real,
    cells: HashMap<PackedCell, T>,
}

impl<T: Default> SpGrid<T> {
    pub fn new(cell_width: Real) -> Self {
        Self {
            cell_width,
            cells: HashMap::new(),
        }
    }

    pub fn cell_width(&self) -> Real {
        self.cell_width
    }

    pub fn get_packed(&self, id: PackedCell) -> Option<&T> {
        self.cells.get(&id)
    }

    pub fn get_packed_mut(&mut self, id: PackedCell) -> &mut T {
        self.cells.entry(id).or_insert_with(T::default)
    }

    pub fn for_each_neighbor_packed_mut<F>(&mut self, base_id: PackedCell, mut f: F)
    where
        F: FnMut(PackedCell, IVec2, &mut T),
    {
        let (ix, iy) = unpack_coords(base_id);
        for (dx, dy) in NEIGHBOR_OFFSETS.iter() {
            let shift = IVec2::new((dx + 1) as i32, (dy + 1) as i32);
            let neighbor_id = pack_coords(ix + dx, iy + dy);
            let cell = self.get_packed_mut(neighbor_id);
            f(neighbor_id, shift, cell);
        }
    }

    pub fn for_each_neighbor_packed<F>(&self, base_id: PackedCell, mut f: F)
    where
        F: FnMut(PackedCell, IVec2, &T),
    {
        let (ix, iy) = unpack_coords(base_id);
        for (dx, dy) in NEIGHBOR_OFFSETS.iter() {
            let shift = IVec2::new((dx + 1) as i32, (dy + 1) as i32);
            let neighbor_id = pack_coords(ix + dx, iy + dy);
            if let Some(cell) = self.cells.get(&neighbor_id) {
                f(neighbor_id, shift, cell);
            }
        }
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (PackedCell, &T)> {
        self.cells.iter().map(|(&id, node)| (id, node))
    }

    pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = (PackedCell, &mut T)> {
        self.cells.iter_mut().map(|(&id, node)| (id, node))
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(PackedCell, &mut T) -> bool,
    {
        self.cells.retain(|&id, node| f(id, node));
    }

    pub fn cell_center(&self, id: PackedCell) -> Vector {
        let (ix, iy) = unpack_coords(id);
        Vector::new(ix as Real * self.cell_width, iy as Real * self.cell_width)
    }
}

impl<T> SpGrid<T> {
    pub const REGION_ID_MASK: u64 = !0;

    pub fn region_neighbors(region_id: PackedCell) -> Vec<PackedCell> {
        let (ix, iy) = unpack_coords(region_id);
        let mut neighbors = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                neighbors.push(pack_coords(ix + dx, iy + dy));
            }
        }
        neighbors
    }
}
