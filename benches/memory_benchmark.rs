use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bevy::prelude::*;
use mpm2d::core::{ParticleRemap, cleanup_grid_cells, zero_grid};
use mpm2d::core::{clear_particle_remap_system, remove_failed_particles_system};
use mpm2d::solver::{grid_to_particle, grid_update, particle_to_grid};
use mpm2d::{GRAVITY, GRID_RESOLUTION, MaterialType, MpmState, Particle, SolverParams};

// Memory tracking allocator
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { System.alloc(layout) };
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn get_memory_usage() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

fn memory_benchmark_system(mut state: ResMut<MpmState>, mut frame_count: Local<u32>) {
    *frame_count += 1;

    if *frame_count == 1 {
        for x in 0..50 {
            for y in 0..100 {
                let position = Vec2::new(x as f32 + 55.0, y as f32 + 20.0);
                let mut particle = Particle::zeroed(MaterialType::water());
                particle.position = position;
                state.add_particle(particle);
            }
        }

        let initial_memory = get_memory_usage();
        println!("Initial memory usage: {} KB", initial_memory / 1024);
    } else if *frame_count == 10 {
        let active_memory = get_memory_usage();
        let grid = state.grid();
        let active_cells = grid.active_cell_count();
        let total_cells = GRID_RESOLUTION * GRID_RESOLUTION;

        println!("Memory after 10 frames: {} KB", active_memory / 1024);
        println!("Active cells: {}/{}", active_cells, total_cells);
        println!(
            "Estimated dense grid memory: {} KB",
            (total_cells * 12) / 1024
        );
        println!("Estimated sparse overhead: HashMap buckets, keys, etc.");

        std::process::exit(0);
    }
}

fn main() {
    let initial_baseline = get_memory_usage();
    println!("Baseline memory: {} KB", initial_baseline / 1024);

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(MpmState::new(SolverParams::default(), GRAVITY))
        .insert_resource(ParticleRemap::default())
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_systems(
            Update,
            (
                memory_benchmark_system,
                zero_grid,
                particle_to_grid,
                cleanup_grid_cells,
                grid_update,
                grid_to_particle,
                remove_failed_particles_system,
                clear_particle_remap_system,
            )
                .chain(),
        )
        .run();
}
