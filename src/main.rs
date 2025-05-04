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
    affine_momentum_matrix: Mat2,
}

impl Particle {
    pub fn zeroed() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0,
            affine_momentum_matrix: Mat2::ZERO,
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
        for y in 0..100 {
            let handle = meshes.add(Circle::new(1.0));

            commands.spawn((
                Particle {
                    position: Vec2 {
                        x: 16.0 + x as f32 / 4.0,
                        y: 64.0 + y as f32 / 4.0,
                    },
                    velocity: Vec2::new(rand.random_range(-10.0..=10.0), rand.random_range(-10.0..=10.0)),
                    mass: 1.0,
                    affine_momentum_matrix: Mat2::ZERO,
                },
                Mesh2d(handle),
                MeshMaterial2d(materials.add(Color::hsl(0.0, 1.0, 0.5))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
    for x in 0..50 {
        for y in 0..100 {
            let handle = meshes.add(Circle::new(1.0));

            commands.spawn((
                Particle {
                    position: Vec2 {
                        x: 112.0 + x as f32 / 4.0,
                        y: 64.0 + y as f32 / 4.0,
                    },
                    velocity: Vec2::new(rand.random_range(-10.0..=10.0), rand.random_range(-10.0..=10.0)),
                    mass: 1.0,
                    affine_momentum_matrix: Mat2::ZERO,
                },
                Mesh2d(handle),
                MeshMaterial2d(materials.add(Color::hsl(0.0, 1.0, 0.5))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
}

fn controls(
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
    window: Query<&Window>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Ok((mut camera, mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let fspeed = 600.0 * time.delta_secs();
    let uspeed = fspeed as u32;
    let window_size = window.resolution.physical_size();

    // Camera movement controls
    if input.pressed(KeyCode::ArrowUp) {
        transform.translation.y += fspeed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= fspeed;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= fspeed;
    }
    if input.pressed(KeyCode::ArrowRight) {
        transform.translation.x += fspeed;
    }

    // Camera zoom controls
    if let Projection::Orthographic(projection2d) = &mut *projection {
        if input.pressed(KeyCode::Comma) {
            projection2d.scale *= 4.0f32.powf(time.delta_secs());
        }

        if input.pressed(KeyCode::Period) {
            projection2d.scale *= 0.25f32.powf(time.delta_secs());
        }
    }

}

// Simulation steps

fn zero_grid(mut grid: ResMut<Grid>) {
    grid.cells.iter_mut().for_each(|cell| cell.zero());
}

fn particle_to_grid_1(query: Query<&Particle>, mut grid: ResMut<Grid>) {
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
                let q = particle.affine_momentum_matrix * cell_distance;

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

const EOS_STIFFNESS: f32 = 10.0;
const EOS_POWER: u8 = 4;

const REST_DENSITY: f32 = 4.0;
const DYNAMIC_VISCOSITY: f32 = 0.1;

fn particle_to_grid_2(time: Res<Time>, query: Query<&Particle>, mut grid: ResMut<Grid>) {
    for particle in query {
        let cell_index = particle.position.as_uvec2();
        let cell_difference = (particle.position - cell_index.as_vec2()) - 0.5;

        let weights: [Vec2; 3] = [
            0.5 * (0.5 - cell_difference).powf(2.0),
            0.75 - cell_difference.powf(2.0),
            0.5 * (0.5 + cell_difference).powf(2.0),
        ];

        let mut density = 0.0;

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);

                let cell_index =
                    cell_position.x as usize * GRID_RESOLUTION + cell_position.y as usize;

                let cell = grid.cells.get_mut(cell_index).unwrap();

                density += cell.mass * weight;
            }
        }

        let volume = particle.mass / density;

        let pressure = f32::max(-0.1, EOS_STIFFNESS * ((density / REST_DENSITY).powi(EOS_POWER as i32) - 1.0));

        let mut stress = Mat2::IDENTITY * -pressure;

        let dudv = particle.affine_momentum_matrix;
        let mut strain = dudv;

        let trace = strain.col(1).x + strain.col(0).y;
        strain.col_mut(0).y = trace;
        strain.col_mut(1).x = trace;

        let viscosity_term = DYNAMIC_VISCOSITY * strain;

        stress += viscosity_term;

        let eq_16_term_0 = -volume * 4.0 * stress * time.delta_secs();

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance = (cell_position.as_vec2() - particle.position) + 0.5;

                let cell_index =
                    cell_position.x as usize * GRID_RESOLUTION + cell_position.y as usize;
                let cell = grid.cells.get_mut(cell_index).unwrap();

                let momentum = eq_16_term_0 * weight * cell_distance;

                cell.velocity += momentum;
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

            if x < 2 || x > GRID_RESOLUTION - 3 {
                cell.velocity.x = 0.0;
            }

            if y < 2 || y > GRID_RESOLUTION - 3 {
                cell.velocity.y = 0.0;
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

        let mut b = Mat2::ZERO;

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

                let term = Mat2::from_cols(weighted_velocity * cell_distance.x, weighted_velocity * cell_distance.y);

                b += term;

                particle.velocity += weighted_velocity;
            }
        }

        particle.affine_momentum_matrix = b * 4.0;

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
                particle_to_grid_1,
                particle_to_grid_2,
                calculate_grid_velocities,
                grid_to_particle,
                update_particle_transforms,
                controls,
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
