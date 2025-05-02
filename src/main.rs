use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

#[derive(Resource)]
struct FrameTimer(Timer);

#[derive(Component)]
struct Cell {
    velocity: Vec2,
    mass: f32,
}

impl Cell {
    pub fn zeroed() -> Self {
        Self {
            velocity: Vec2::ZERO,
            mass: 0.0,
        }
    }

    pub fn zero(&mut self) {
        self.velocity = Vec2::ZERO;
        self.mass = 0.0;
    }
}

#[derive(Resource)]
struct Grid {
    cells: Vec<Cell>,
}

const GRID_RESOLUTION: usize = 128;
const GRAVITY: Vec2 = Vec2::new(0.0, -20.0);

fn init_grid(mut grid: ResMut<Grid>) {
    grid.cells.clear();
    grid.cells.reserve_exact(GRID_RESOLUTION * GRID_RESOLUTION);
    for _ in 0..(GRID_RESOLUTION * GRID_RESOLUTION) {
        grid.cells.push(Cell::zeroed());
    }
}

#[derive(Component)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    mass: f32,
}

impl Particle {
    pub fn zeroed() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0,
        }
    }
}

fn init_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rand = rand::rng();
    for x in 0..50 {
        for y in 0..50 {
            let handle = meshes.add(Circle::new(1.0));

            commands.spawn((
                Particle {
                    position: Vec2 {
                        x: 64.0 + x as f32 / 4.0,
                        y: 64.0 + y as f32 / 4.0,
                    },
                    velocity: Vec2::new(rand.random_range(-1000.0..=1000.0), rand.random_range(-1000.0..=1000.0)),
                    mass: 1.0,
                },
                Mesh2d(handle),
                MeshMaterial2d(materials.add(Color::hsl(0.0, 1.0, 0.5))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
}

// Simulation steps

fn zero_grid(mut grid: ResMut<Grid>) {
    grid.cells.iter_mut().for_each(|cell| cell.zero());
}

fn particle_to_grid(query: Query<&Particle>, mut grid: ResMut<Grid>) {
    for particle in query {
        let cell_index = particle.position.as_uvec2();
        let cell_difference = (particle.position - cell_index.as_vec2()) - 0.5;

        let weights: [Vec2; 3] = [
            0.5 * (0.5 - cell_difference).powf(2.0),
            0.75 - cell_difference.powf(2.0),
            0.5 * (0.5 + cell_difference).powf(2.0),
        ];

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance =
                    (cell_position.as_vec2() - particle.position) + 0.5;
                let q = Mat2::ZERO * cell_distance; // TODO: Should use the affine momentum matrix in Particle

                let mass_contribution = weight * particle.mass;

                let cell_index =
                    cell_position.x as usize * GRID_RESOLUTION + cell_position.y as usize;

                let cell = grid.cells.get_mut(cell_index).unwrap();

                cell.mass += mass_contribution;

                cell.velocity += mass_contribution * (particle.velocity + q);
            }
        }
    }
}

fn calculate_grid_velocities(time: Res<Time>, mut grid: ResMut<Grid>) {
    for (index, cell) in grid.cells.iter_mut().enumerate() {
        if cell.mass > 0.0 {
            let gravity_velocity = time.delta_secs() * GRAVITY;
            cell.velocity /= cell.mass;
            cell.velocity += gravity_velocity;

            let x = index / GRID_RESOLUTION;
            let y = index % GRID_RESOLUTION;

            if x < 2 {
                cell.velocity.x = 100.1;
            }

            if x > GRID_RESOLUTION - 3 {
                cell.velocity.x = -100.1;
            }

            if y < 2 {
                cell.velocity.x *= 0.9;
                cell.velocity.y = 0.0;
            }

            if y > GRID_RESOLUTION - 3 {
                cell.velocity.y = -100.1;
            }
        }
    }
}

fn grid_to_particle(time: Res<Time>, query: Query<&mut Particle>, grid: Res<Grid>) {
    for mut particle in query {
        particle.velocity = Vec2::ZERO;

        let cell_index = particle.position.as_uvec2();
        let cell_difference = (particle.position - cell_index.as_vec2()) - 0.5;

        let weights: [Vec2; 3] = [
            0.5 * (0.5 - cell_difference).powf(2.0),
            0.75 - cell_difference.powf(2.0),
            0.5 * (0.5 + cell_difference).powf(2.0),
        ];

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_index =
                    cell_position.x as usize * GRID_RESOLUTION + cell_position.y as usize;

                let cell_distance =
                    (cell_position.as_vec2() - particle.position) + 0.5;
                let weighted_velocity = grid.cells.get(cell_index).unwrap().velocity * weight;

                particle.velocity += weighted_velocity;
            }
        }

        let particle_velocity = particle.velocity;

        particle.position += particle_velocity * time.delta_secs();

        particle.position = particle
            .position
            .clamp(Vec2::splat(1.0), Vec2::splat(GRID_RESOLUTION as f32 - 2.0));
    }
}

fn update_particle_transforms(query: Query<(&mut Transform, &Particle)>) {
    for (mut transform, particle) in query {
        transform.translation = Vec3::new((particle.position.x - 64.0) * 4.0, (particle.position.y - 64.0) * 4.0, 0.0);
    }
}

pub struct MpmPlugin;

impl Plugin for MpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid { cells: Vec::new() });
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 60.0)));
        app.add_systems(Startup, (init_grid, init_particles).chain());
        app.add_systems(
            FixedUpdate,
            (
                zero_grid,
                particle_to_grid,
                calculate_grid_velocities,
                grid_to_particle,
                update_particle_transforms
            )
                .chain(),
        );
    }
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MpmPlugin)
        .add_systems(Startup, init)
        .run();
}
