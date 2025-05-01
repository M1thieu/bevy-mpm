use bevy::prelude::*;

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

impl Grid {
    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(x + y * GRID_RESOLUTION)
    }
}

const GRID_RESOLUTION: usize = 32;

fn init_grid(mut grid: ResMut<Grid>) {
    grid.cells.clear();
    grid.cells.reserve_exact(GRID_RESOLUTION);
    grid.cells.push(Cell::zeroed());
}

fn zero_grid(mut grid: ResMut<Grid>) {
    grid.cells.iter_mut().for_each(|cell| cell.zero());
}

fn calculate_grid_velocities(mut grid: ResMut<Grid>) {}

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
    for i in 0..10 {
        let handle = meshes.add(Circle::new(1.0));

        commands.spawn((
            Particle { position: Vec2 { x: i as f32 * 5.0, y: 0.0 }, velocity: Vec2::ZERO, mass: 1.0 },
            Mesh2d(handle),
            MeshMaterial2d(materials.add(Color::hsl(0.0, 1.0, 0.5))),
            Transform::from_xyz(0.0, 0.0, 0.0)
        ));
    }
}

fn particle_to_grid(query: Query<&mut Particle>, mut grid: ResMut<Grid>) {
    for mut particle in query {
        particle.velocity += Vec2 { x: 0.0, y: -0.1 };
    }
}

fn grid_to_particle(query: Query<&mut Particle>, grid: Res<Grid>) {
    for mut particle in query {
        let particle_velocity = particle.velocity;
        particle.position += particle_velocity;
    }
}

fn update_particle_transforms(query: Query<(&mut Transform, &Particle)>) {
    for (mut transform, particle) in query {
        transform.translation = Vec3::new(particle.position.x, particle.position.y, 0.0);
    }
}

pub struct MpmPlugin;

impl Plugin for MpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid { cells: Vec::new() });
        app.add_systems(Startup, (init_grid, init_particles).chain());
        app.add_systems(
            Update,
            (
                zero_grid,
                particle_to_grid,
                calculate_grid_velocities,
                grid_to_particle,
                update_particle_transforms,
            )
                .chain(),
        );
    }
}

fn init(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MpmPlugin)
        .add_systems(Startup, init)
        .run();
}
