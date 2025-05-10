use bevy::prelude::*;
use crate::grid::{Grid, GRID_RESOLUTION};
use crate::solver::Particle;
use std::time::Instant;

/// Configuration for the bukkit system
#[derive(Resource, Clone)]
pub struct BukkitConfig {
    pub size: usize,
    pub halo: usize,
    pub capacity_hint: usize,
}

impl Default for BukkitConfig {
    fn default() -> Self {
        Self {
            size: 8,
            halo: 1,
            capacity_hint: 64,
        }
    }
}

/// Thread group data structure - minimal version of EA's BukkitThreadData
#[derive(Clone, Debug)]
pub struct BukkitThreadData {
    pub bukkit_index: usize,
    pub bukkit_x: usize,
    pub bukkit_y: usize,
    pub range_start: usize,    // Start index in allocated buffer
    pub range_count: usize,    // Number of particles in this bukkit
}

/// Core data structure for the bukkit spatial partitioning system
#[derive(Resource)]
pub struct BukkitSystem {
    pub count_x: usize,
    pub count_y: usize,
    pub particle_indices: Vec<Vec<Entity>>,
    pub active_grid_cells: Vec<usize>,
    pub thread_data: Vec<BukkitThreadData>,
    pub particle_counts: Vec<usize>,           // Count per bukkit
    pub allocated_indices: Vec<Entity>,         // Pre-allocated contiguous buffer
}

impl BukkitSystem {
    pub fn new(config: &BukkitConfig) -> Self {
        let count_x = (GRID_RESOLUTION + config.size - 1) / config.size;
        let count_y = (GRID_RESOLUTION + config.size - 1) / config.size;
        let total_bukkits = count_x * count_y;
        
        Self {
            count_x,
            count_y,
            particle_indices: vec![Vec::with_capacity(config.capacity_hint); total_bukkits],
            active_grid_cells: Vec::with_capacity(GRID_RESOLUTION * GRID_RESOLUTION / 4),
            thread_data: Vec::with_capacity(total_bukkits),
            particle_counts: vec![0; total_bukkits],
            allocated_indices: Vec::new(),
        }
    }
    
    #[inline]
    pub fn mark_grid_cell_active(&mut self, cell_idx: usize) {
        self.active_grid_cells.push(cell_idx);
    }
    
    pub fn get_grid_range(&self, bukkit_idx: usize, config: &BukkitConfig) -> (usize, usize, usize, usize) {
        let bukkit_x = bukkit_idx % self.count_x;
        let bukkit_y = bukkit_idx / self.count_x;
        
        let min_grid_x = bukkit_x.saturating_mul(config.size).saturating_sub(config.halo);
        let min_grid_y = bukkit_y.saturating_mul(config.size).saturating_sub(config.halo);
        let max_grid_x = ((bukkit_x + 1) * config.size + config.halo).min(GRID_RESOLUTION);
        let max_grid_y = ((bukkit_y + 1) * config.size + config.halo).min(GRID_RESOLUTION);
        
        (min_grid_x, min_grid_y, max_grid_x, max_grid_y)
    }
}

#[inline]
pub fn position_to_bukkit_id(position: Vec2, bukkit_size: usize) -> UVec2 {
    UVec2::new(
        (position.x as usize / bukkit_size) as u32,
        (position.y as usize / bukkit_size) as u32
    )
}

#[inline]
pub fn bukkit_address_to_index(address: UVec2, bukkit_count_x: usize) -> usize {
    address.y as usize * bukkit_count_x + address.x as usize
}

// Phase 1: Count particles per bukkit
pub fn count_particles_per_bukkit(
    query: Query<&Particle>,
    mut bukkits: ResMut<BukkitSystem>,
    config: Res<BukkitConfig>,
) {
    let start = Instant::now();
    
    // Reset counts
    for count in &mut bukkits.particle_counts {
        *count = 0;
    }
    
    // Count particles per bukkit
    for particle in &query {
        let bukkit_pos = position_to_bukkit_id(particle.position, config.size);
        
        if bukkit_pos.x < bukkits.count_x as u32 && bukkit_pos.y < bukkits.count_y as u32 {
            let bukkit_idx = bukkit_address_to_index(bukkit_pos, bukkits.count_x);
            bukkits.particle_counts[bukkit_idx] += 1;
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("bukkit_count: {:.3}ms", elapsed);
}

// Phase 2: Allocate contiguous memory for particle indices
pub fn allocate_bukkit_memory(
    mut bukkits: ResMut<BukkitSystem>,
) {
    let start = Instant::now();
    
    // Calculate total needed capacity
    let total_particles: usize = bukkits.particle_counts.iter().sum();
    
    // Allocate contiguous buffer
    bukkits.allocated_indices.clear();
    bukkits.allocated_indices.resize(total_particles, Entity::PLACEHOLDER);
    
    // Extract values to avoid borrowing conflicts
    let count_x = bukkits.count_x;
    let particle_counts = bukkits.particle_counts.clone();
    
    // Generate thread data with pre-calculated ranges
    bukkits.thread_data.clear();
    let mut range_start = 0;
    
    for (bukkit_idx, &count) in particle_counts.iter().enumerate() {
        if count > 0 {
            bukkits.thread_data.push(BukkitThreadData {
                bukkit_index: bukkit_idx,
                bukkit_x: bukkit_idx % count_x,
                bukkit_y: bukkit_idx / count_x,
                range_start,
                range_count: count,
            });
            
            range_start += count;
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("bukkit_allocate: {:.3}ms", elapsed);
}

// Phase 3: Insert particles into allocated ranges
pub fn insert_particles_to_bukkits(
    query: Query<(Entity, &Particle)>,
    mut bukkits: ResMut<BukkitSystem>,
    config: Res<BukkitConfig>,
) {
    let start = Instant::now();
    
    // Reset particle indices
    bukkits.active_grid_cells.clear();
    for indices in &mut bukkits.particle_indices {
        indices.clear();
    }
    
    // Insert particles into pre-allocated ranges
    for (entity, particle) in &query {
        let bukkit_pos = position_to_bukkit_id(particle.position, config.size);
        
        if bukkit_pos.x < bukkits.count_x as u32 && bukkit_pos.y < bukkits.count_y as u32 {
            let bukkit_idx = bukkit_address_to_index(bukkit_pos, bukkits.count_x);
            
            // Find the thread data for this bukkit
            if let Some(thread_data) = bukkits.thread_data.iter().find(|td| td.bukkit_index == bukkit_idx) {
                let insert_pos = thread_data.range_start + bukkits.particle_indices[bukkit_idx].len();
                if insert_pos < thread_data.range_start + thread_data.range_count {
                    bukkits.allocated_indices[insert_pos] = entity;
                    bukkits.particle_indices[bukkit_idx].push(entity);
                }
            }
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("bukkit_insert: {:.3}ms", elapsed);
}

pub fn selective_grid_clear(
    mut grid: ResMut<Grid>,
    bukkits: Res<BukkitSystem>,
) {
    let start = Instant::now();
    
    for &cell_idx in &bukkits.active_grid_cells {
        if cell_idx < grid.cells.len() {
            grid.cells[cell_idx].zero();
        }
    }
    
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    info!("selective_grid_clear: {:.3}ms", elapsed);
}