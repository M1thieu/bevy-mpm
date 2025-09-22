use std::time::Duration;
use bevy::prelude::*;
use mpm2d::core::{calculate_grid_velocities, zero_grid};
use mpm2d::solver::{grid_to_particle, particle_to_grid_forces, particle_to_grid_mass_velocity};
use mpm2d::{GRAVITY, GRID_RESOLUTION, Grid, MaterialType, Particle, SolverParams};

fn create_particles(mut commands: Commands) {
    println!("Creating 5000 particles...");
    for x in 0..50 {
        for y in 0..100 {
            let position = Vec2::new(x as f32 + 55.0, y as f32 + 20.0);
            commands.spawn(Particle {
                position,
                velocity: Vec2::ZERO,
                mass: 1.0,
                material_type: MaterialType::Water {
                    vp0: 1.0,
                    ap: 0.0,
                    jp: 1.0,
                },
                grid_index: 0,
                deformation_gradient: Mat2::IDENTITY,
                velocity_gradient: Mat2::ZERO,
                affine_momentum_matrix: Mat2::ZERO,
                failed: false,
                condition_number: 1.0,
                volume0: 1.0,
            });
        }
    }
}

fn memory_tracker(
    grid: Res<Grid>,
    particles: Query<&Particle>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;

    if *frame_count % 60 == 0 {  // Every second at 60fps
        let active_cells = grid.active_cell_count();
        let total_cells = GRID_RESOLUTION * GRID_RESOLUTION;
        let particle_count = particles.iter().count();

        // Estimate memory usage
        let dense_grid_bytes = total_cells * std::mem::size_of::<mpm2d::Cell>();
        let sparse_cells_bytes = active_cells * std::mem::size_of::<mpm2d::Cell>();
        let hashmap_overhead = active_cells * (std::mem::size_of::<(i32, i32)>() + 8); // rough estimate
        let particles_bytes = particle_count * std::mem::size_of::<Particle>();

        println!("\n--- Memory Analysis (Frame {}) ---", *frame_count);
        println!("Particles: {}", particle_count);
        println!("Active cells: {}/{}", active_cells, total_cells);
        println!("Dense grid would use: {} KB", dense_grid_bytes / 1024);
        println!("Sparse cells data: {} KB", sparse_cells_bytes / 1024);
        println!("HashMap overhead: {} KB", hashmap_overhead / 1024);
        println!("Total sparse grid: {} KB", (sparse_cells_bytes + hashmap_overhead) / 1024);
        println!("Particles memory: {} KB", particles_bytes / 1024);

        let savings = dense_grid_bytes as f32 - (sparse_cells_bytes + hashmap_overhead) as f32;
        let savings_percent = (savings / dense_grid_bytes as f32) * 100.0;
        println!("Estimated real savings: {:.1} KB ({:.1}%)", savings / 1024.0, savings_percent);
    }

    if *frame_count > 300 {  // Exit after 5 seconds
        std::process::exit(0);
    }
}

fn main() {
    println!("Memory Test: Sparse Grid vs Dense Grid");
    println!("Dense grid theoretical size: {} KB", (GRID_RESOLUTION * GRID_RESOLUTION * std::mem::size_of::<mpm2d::Cell>()) / 1024);

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(Grid::new())
        .insert_resource(SolverParams::default())
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 60.0)))
        .add_systems(Startup, create_particles)
        .add_systems(
            FixedUpdate,
            (
                zero_grid,
                particle_to_grid_mass_velocity,
                particle_to_grid_forces,
                |time: Res<Time>, grid: ResMut<Grid>| {
                    calculate_grid_velocities(time, grid, GRAVITY);
                },
                grid_to_particle,
                memory_tracker,
            ).chain(),
        )
        .run();
}