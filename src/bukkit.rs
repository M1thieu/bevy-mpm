use bevy::prelude::*;
use crate::grid::{Grid, GRID_RESOLUTION};
use crate::solver::Particle;

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

/// Core data structure for the bukkit spatial partitioning system
#[derive(Resource)]
pub struct BukkitSystem {
    pub count_x: usize,
    pub count_y: usize,
    pub particle_indices: Vec<Vec<Entity>>,
    pub active_grid_cells: Vec<usize>,
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
        }
    }
    
    /// Track a grid cell as active (used this frame)
    #[inline]
    pub fn mark_grid_cell_active(&mut self, cell_idx: usize) {
        self.active_grid_cells.push(cell_idx);
    }
    
    /// Get the grid range for a given bukkit (including halo)
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

/// Convert particle position to bukkit index
#[inline]
pub fn position_to_bukkit_id(position: Vec2, bukkit_size: usize) -> UVec2 {
    UVec2::new(
        (position.x as usize / bukkit_size) as u32,
        (position.y as usize / bukkit_size) as u32
    )
}

/// Convert bukkit coordinates to linear index
#[inline]
pub fn bukkit_address_to_index(address: UVec2, bukkit_count_x: usize) -> usize {
    address.y as usize * bukkit_count_x + address.x as usize
}

/// System to assign particles to bukkits
pub fn assign_particles_to_bukkits(
    query: Query<(Entity, &Particle)>,
    mut bukkits: ResMut<BukkitSystem>,
    config: Res<BukkitConfig>,
) {
    // Clear previous assignments
    for indices in &mut bukkits.particle_indices {
        indices.clear();
    }
    
    // Assign particles to bukkits
    for (entity, particle) in query.iter() {
        let bukkit_pos = position_to_bukkit_id(particle.position, config.size);
        
        if bukkit_pos.x < bukkits.count_x as u32 && bukkit_pos.y < bukkits.count_y as u32 {
            let bukkit_idx = bukkit_address_to_index(bukkit_pos, bukkits.count_x);
            bukkits.particle_indices[bukkit_idx].push(entity);
        }
    }
    
    // Clear active grid cells for the next frame
    bukkits.active_grid_cells.clear();
}

/// Selectively clear only the grid cells that were used
pub fn selective_grid_clear(
    mut grid: ResMut<Grid>,
    bukkits: Res<BukkitSystem>,
) {
    for &cell_idx in &bukkits.active_grid_cells {
        if cell_idx < grid.cells.len() {
            grid.cells[cell_idx].zero();
        }
    }
}