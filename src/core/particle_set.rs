use std::collections::HashSet;
use std::ops::Range;

use crate::core::Particle;
use crate::math::{Real, Vector};

pub type PackedCell = u64;

fn pack_coords(ix: i32, iy: i32) -> PackedCell {
    ((ix as u64) << 32) | (iy as u32 as u64)
}

#[derive(Clone)]
pub struct ParticleSet {
    pub particles: Vec<Particle>,
    pub order: Vec<usize>,
    pub regions: Vec<(PackedCell, Range<usize>)>,
    pub active_regions: HashSet<PackedCell>,
    pub active_cells: Vec<PackedCell>,
    pub particle_bins: Vec<[usize; 4]>,
}

impl ParticleSet {
    pub fn len(&self) -> usize {
        self.particles.len()
    }

    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            order: Vec::new(),
            regions: Vec::new(),
            active_regions: HashSet::new(),
            active_cells: Vec::new(),
            particle_bins: Vec::new(),
        }
    }


    pub fn iter(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Particle> {
        self.particles.iter_mut()
    }

    pub fn insert(&mut self, particle: Particle) -> usize {
        let index = self.particles.len();
        self.order.push(index);
        self.particles.push(particle);
        index
    }

    pub fn insert_batch(&mut self, mut batch: Vec<Particle>) {
        let start = self.order.len();
        self.order.extend(start..start + batch.len());
        self.particles.append(&mut batch);
    }

    pub fn push(&mut self, particle: Particle) -> usize {
        self.insert(particle)
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn particles_mut(&mut self) -> &mut [Particle] {
        &mut self.particles
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Particle> {
        self.particles.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Particle> {
        self.particles.get_mut(index)
    }

    pub fn remove_failed(&mut self) -> Vec<Option<usize>> {
        if !self.particles.iter().any(|particle| particle.failed) {
            return Vec::new();
        }

        let old_len = self.particles.len();
        let mut mapping = vec![None; old_len];
        let mut survivors = Vec::with_capacity(old_len);

        for (old_idx, particle) in self.particles.drain(..).enumerate() {
            if !particle.failed {
                let new_idx = survivors.len();
                mapping[old_idx] = Some(new_idx);
                survivors.push(particle);
            }
        }

        self.particles = survivors;
        mapping
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.order.clear();
        self.regions.clear();
        self.active_regions.clear();
        self.active_cells.clear();
        self.particle_bins.clear();
    }

    pub fn rebuild_bins(&mut self, cell_width: Real) {
        if self.particles.is_empty() {
            self.order.clear();
            self.regions.clear();
            self.active_regions.clear();
            self.active_cells.clear();
            self.particle_bins.clear();
            return;
        }

        if self.order.len() != self.particles.len() {
            self.order = (0..self.particles.len()).collect();
        }

        for (idx, particle) in self.particles.iter_mut().enumerate() {
            let grid_coords = grid_coords(particle.position, cell_width);
            particle.grid_index = pack_coords(grid_coords.0, grid_coords.1);
            if idx >= self.active_cells.len() {
                self.active_cells.push(particle.grid_index);
            } else {
                self.active_cells[idx] = particle.grid_index;
            }
        }

        self.order.sort_by_key(|&idx| self.particles[idx].grid_index);

        self.particle_bins.clear();
        self.regions.clear();
        self.active_regions.clear();

        let mut current_region = self.particles[self.order[0]].grid_index;
        let mut range_start = 0;
        let mut current_bin = [usize::MAX; 4];
        let mut bin_len = 0;

        for (sorted_idx, particle_idx) in self.order.iter().enumerate() {
            let particle = &self.particles[*particle_idx];
            if particle.grid_index != current_region {
                self.regions.push((current_region, range_start..sorted_idx));
                self.active_regions.insert(current_region);
                range_start = sorted_idx;
                current_region = particle.grid_index;
            }

            if bin_len == 4 {
                self.particle_bins.push(current_bin);
                current_bin = [usize::MAX; 4];
                bin_len = 0;
            }

            current_bin[bin_len] = *particle_idx;
            bin_len += 1;
        }

        if bin_len > 0 {
            self.particle_bins.push(current_bin);
        }

        self.regions.push((current_region, range_start..self.order.len()));
        self.active_regions.insert(current_region);
    }
}

fn grid_coords(position: Vector, cell_width: Real) -> (i32, i32) {
    let inv = 1.0 / cell_width;
    let ix = (position.x * inv).round() as i32;
    let iy = (position.y * inv).round() as i32;
    (ix, iy)
}
