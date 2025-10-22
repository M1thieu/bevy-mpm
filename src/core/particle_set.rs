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
    particles: Vec<Particle>,
    order: Vec<usize>,
    regions: Vec<(PackedCell, Range<usize>)>,
    active_regions: HashSet<PackedCell>,
    active_cells: Vec<PackedCell>,
    particle_bins: Vec<[usize; 4]>,
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
        self.particles.push(particle);
        self.invalidate_spatial_index();
        index
    }

    pub fn insert_batch(&mut self, mut batch: Vec<Particle>) {
        self.particles.append(&mut batch);
        self.invalidate_spatial_index();
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

    pub fn particle_order(&self) -> &[usize] {
        &self.order
    }

    pub fn cell_regions(&self) -> &[(PackedCell, Range<usize>)] {
        &self.regions
    }

    pub fn active_region_ids(&self) -> &HashSet<PackedCell> {
        &self.active_regions
    }

    pub fn cell_assignments(&self) -> &[PackedCell] {
        &self.active_cells
    }

    pub fn bins(&self) -> &[[usize; 4]] {
        &self.particle_bins
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
        self.invalidate_spatial_index();
        mapping
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.invalidate_spatial_index();
    }

    pub fn rebuild_bins(&mut self, cell_width: Real) {
        let particle_count = self.particles.len();
        if particle_count == 0 {
            self.invalidate_spatial_index();
            return;
        }

        self.order.clear();
        self.order.extend(0..particle_count);
        self.active_regions.clear();
        self.regions.clear();
        self.particle_bins.clear();
        self.active_cells.resize(particle_count, 0);

        for (idx, particle) in self.particles.iter_mut().enumerate() {
            let (ix, iy) = grid_coords(particle.position, cell_width);
            let packed = pack_coords(ix, iy);
            particle.grid_index = packed;
            self.active_cells[idx] = packed;
        }

        self.order
            .sort_by_key(|&idx| self.particles[idx].grid_index);

        let mut current_region: Option<(PackedCell, usize)> = None;
        let mut current_bin = [usize::MAX; 4];
        let mut bin_len = 0;

        for (sorted_idx, &particle_idx) in self.order.iter().enumerate() {
            let particle = &self.particles[particle_idx];
            let cell = particle.grid_index;

            match current_region {
                Some((region_cell, start_idx)) if region_cell != cell => {
                    self.regions.push((region_cell, start_idx..sorted_idx));
                    self.active_regions.insert(region_cell);
                    current_region = Some((cell, sorted_idx));
                }
                None => {
                    current_region = Some((cell, sorted_idx));
                }
                _ => {}
            }

            if bin_len == 4 {
                self.particle_bins.push(current_bin);
                current_bin = [usize::MAX; 4];
                bin_len = 0;
            }

            current_bin[bin_len] = particle_idx;
            bin_len += 1;
        }

        if bin_len > 0 {
            self.particle_bins.push(current_bin);
        }

        if let Some((cell, start_idx)) = current_region {
            self.regions.push((cell, start_idx..self.order.len()));
            self.active_regions.insert(cell);
        }
    }

    fn invalidate_spatial_index(&mut self) {
        self.order.clear();
        self.regions.clear();
        self.active_regions.clear();
        self.active_cells.clear();
        self.particle_bins.clear();
    }
}

fn grid_coords(position: Vector, cell_width: Real) -> (i32, i32) {
    let inv = 1.0 / cell_width;
    let ix = (position.x * inv).round() as i32;
    let iy = (position.y * inv).round() as i32;
    (ix, iy)
}
