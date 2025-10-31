/// Simple custom benchmarking without criterion
/// Avoids Windows MSVC linker issues with rayon/criterion
use std::time::Instant;
use bevy::prelude::*;
use mpm2d::{MpmState, SolverParams, Particle, MaterialType, GRAVITY};

fn time_it<F: FnMut()>(name: &str, iterations: usize, mut f: F) {
    // Warmup
    for _ in 0..5 {
        f();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();

    let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    println!("{}: {:.3}ms avg ({} iterations)", name, avg_ms, iterations);
}

fn create_test_particles(count: usize) -> Vec<Particle> {
    let side = (count as f32).sqrt() as usize;
    let mut particles = Vec::new();

    for x in 0..side {
        for y in 0..side {
            if particles.len() >= count {
                break;
            }
            let position = Vec2::new(x as f32 + 16.0, y as f32 + 32.0);
            let mut particle = Particle::zeroed(MaterialType::water());
            particle.position = position;
            particle.velocity = Vec2::new(1.0, -2.0);
            particles.push(particle);
        }
    }

    particles
}

fn main() {
    println!("\n=== MPM2D Benchmarks ===\n");

    // Benchmark particle binning with different sizes
    println!("--- Particle Binning ---");
    for &count in &[1000, 5000, 10000, 20000] {
        let mut state = MpmState::new(SolverParams::default(), GRAVITY);
        let particles = create_test_particles(count);
        for p in particles {
            state.add_particle(p);
        }

        time_it(
            &format!("rebuild_bins (n={})", count),
            20,
            || {
                state.rebuild_particle_bins();
            },
        );
    }

    println!("\n--- Grid Operations ---");
    for &count in &[1000, 5000, 10000, 20000] {
        let mut state = MpmState::new(SolverParams::default(), GRAVITY);
        let particles = create_test_particles(count);
        for p in particles {
            state.add_particle(p);
        }
        state.rebuild_particle_bins();

        time_it(
            &format!("zero_grid (n={})", count),
            50,
            || {
                state.zero_grid();
            },
        );
    }

    println!("\n--- Combined Operations ---");
    for &count in &[1000, 5000, 10000] {
        let mut state = MpmState::new(SolverParams::default(), GRAVITY);
        let particles = create_test_particles(count);
        for p in particles {
            state.add_particle(p);
        }

        time_it(
            &format!("bin+zero (n={})", count),
            10,
            || {
                state.rebuild_particle_bins();
                state.zero_grid();
            },
        );
    }

    println!("\n=== Benchmark Complete ===\n");
}
