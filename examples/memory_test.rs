use bevy::prelude::*;
use mpm2d::core::{ParticleRemap, cleanup_grid_cells, zero_grid};
use mpm2d::core::{clear_particle_remap_system, remove_failed_particles_system};
use mpm2d::solver::{grid_to_particle, grid_update, particle_to_grid};
use mpm2d::{GRAVITY, GRID_RESOLUTION, MaterialType, MpmState, Particle, SolverParams};
use std::time::Duration;

fn create_particles(mut state: ResMut<MpmState>) {
    println!("Creating 5000 particles...");
    for x in 0..50 {
        for y in 0..100 {
            let position = Vec2::new(x as f32 + 55.0, y as f32 + 20.0);
            let mut particle = Particle::zeroed(MaterialType::water());
            particle.position = position;
            state.add_particle(particle);
        }
    }
}

fn memory_tracker(state: Res<MpmState>, mut frame_count: Local<u32>) {
    *frame_count += 1;

    if *frame_count % 60 == 0 {
        let grid = state.grid();
        let active_cells = grid.active_cell_count();
        let total_cells = GRID_RESOLUTION * GRID_RESOLUTION;
        let particle_count = state.particle_count();

        let dense_grid_bytes = total_cells * std::mem::size_of::<mpm2d::GridNode>();
        let sparse_cells_bytes = active_cells * std::mem::size_of::<mpm2d::GridNode>();
        let hashmap_overhead = active_cells * (std::mem::size_of::<(i32, i32)>() + 8);
        let particles_bytes = particle_count * std::mem::size_of::<Particle>();

        println!("\n--- Memory Analysis (Frame {}) ---", *frame_count);
        println!("Particles: {}", particle_count);
        println!("Active cells: {}/{}", active_cells, total_cells);
        println!("Dense grid would use: {} KB", dense_grid_bytes / 1024);
        println!("Sparse cells data: {} KB", sparse_cells_bytes / 1024);
        println!("HashMap overhead: {} KB", hashmap_overhead / 1024);
        println!(
            "Total sparse grid: {} KB",
            (sparse_cells_bytes + hashmap_overhead) / 1024
        );
        println!("Particles memory: {} KB", particles_bytes / 1024);

        let savings = dense_grid_bytes as f32 - (sparse_cells_bytes + hashmap_overhead) as f32;
        let savings_percent = (savings / dense_grid_bytes as f32) * 100.0;
        println!(
            "Estimated real savings: {:.1} KB ({:.1}%)",
            savings / 1024.0,
            savings_percent
        );
    }

    if *frame_count > 300 {
        std::process::exit(0);
    }
}

fn main() {
    println!("Memory Test: Sparse Grid vs Dense Grid");
    println!(
        "Dense grid theoretical size: {} KB",
        (GRID_RESOLUTION * GRID_RESOLUTION * std::mem::size_of::<mpm2d::GridNode>()) / 1024
    );

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(MpmState::new(SolverParams::default(), GRAVITY))
        .insert_resource(ParticleRemap::default())
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_systems(Startup, create_particles)
        .add_systems(
            FixedUpdate,
            (
                zero_grid,
                particle_to_grid,
                cleanup_grid_cells,
                grid_update,
                grid_to_particle,
                remove_failed_particles_system,
                clear_particle_remap_system,
                memory_tracker,
            )
                .chain(),
        )
        .run();
}
