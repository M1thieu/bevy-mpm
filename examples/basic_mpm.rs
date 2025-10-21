// Minimal MLS-MPM example using the new resource-driven solver state.
use std::time::Duration;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use mpm2d::core::{
    cleanup_grid_cells, remove_failed_particles_system, zero_grid, GridInterpolation, MpmState,
    ParticleRemap,
};
use mpm2d::solver::{grid_to_particle, grid_update, particle_to_grid};
use mpm2d::{GRAVITY, MaterialType, Particle, SolverParams};
use rand::Rng;

const CLUSTER_ORIGINS: [Vec2; 2] = [Vec2::new(16.0, 32.0), Vec2::new(112.0, 32.0)];
const CLUSTER_WIDTH: u32 = 45;
const CLUSTER_HEIGHT: u32 = 90;

#[derive(Component)]
struct ParticleVisual {
    index: usize,
}

fn sim_to_world(position: Vec2) -> Vec3 {
    Vec3::new((position.x - 64.0) * 4.0, (position.y - 64.0) * 4.0, 0.0)
}

fn init_grid(_state: ResMut<MpmState>) {
    // Sparse grid is created by MpmState::new
}

fn spawn_particle_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    index: usize,
    position: Vec2,
    color: Color,
) {
    commands.spawn((
        ParticleVisual { index },
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(sim_to_world(position)),
    ));
}

fn init_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<MpmState>,
) {
    let mut rand = rand::rng();

    for (cluster_index, origin) in CLUSTER_ORIGINS.iter().enumerate() {
        for x in 0..CLUSTER_WIDTH {
            for y in 0..CLUSTER_HEIGHT {
                let mut particle = Particle::zeroed(MaterialType::water());
                particle.position = Vec2 {
                    x: origin.x + x as f32 / 4.0,
                    y: origin.y + y as f32 / 4.0,
                };
                particle.velocity = Vec2::new(
                    rand.random_range(-1.0..=1.0),
                    rand.random_range(-1.0..=1.0),
                );

                let position = particle.position;
                let index = state.add_particle(particle);
                spawn_particle_entity(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    index,
                    position,
                    Color::hsl(210.0, 0.7, 0.3 + cluster_index as f32 * 0.05),
                );
            }
        }
    }
}

fn controls(
    mut commands: Commands,
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
    window: Query<&Window>,
    input: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<MpmState>,
) {
    let Ok(_window) = window.single() else {
        return;
    };
    let Ok((_camera, mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let fspeed = 600.0 * time.delta_secs();

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

    if mouse.pressed(MouseButton::Left) {
        let mut rand = rand::rng();
        let position = Vec2 {
            x: 64.0 + rand.random_range(-2.0..=2.0),
            y: 64.0 + rand.random_range(-2.0..=2.0),
        };
        let velocity = Vec2::new(
            rand.random_range(-12.0..=12.0),
            rand.random_range(-40.0..=-10.0),
        );

        let mut particle = Particle::zeroed(MaterialType::water());
        particle.position = position;
        particle.velocity = velocity;
        let position = particle.position;
        let index = state.add_particle(particle);

        spawn_particle_entity(
            &mut commands,
            &mut meshes,
            &mut materials,
            index,
            position,
            Color::hsl(0.0, 1.0, 0.5),
        );
    }

    if let Projection::Orthographic(projection2d) = &mut *projection {
        if input.pressed(KeyCode::Comma) {
            projection2d.scale *= 4.0f32.powf(time.delta_secs());
        }

        if input.pressed(KeyCode::Period) {
            projection2d.scale *= 0.25f32.powf(time.delta_secs());
        }
    }
}

fn update_particle_transforms(
    state: Res<MpmState>,
    mut query: Query<(&ParticleVisual, &mut Transform)>,
) {
    let particles = state.particles();
    for (visual, mut transform) in query.iter_mut() {
        if let Some(particle) = particles.get(visual.index) {
            transform.translation = sim_to_world(particle.position);
        }
    }
}

fn apply_particle_remap(
    mut commands: Commands,
    remap: Res<ParticleRemap>,
    mut visuals: Query<(Entity, &mut ParticleVisual)>,
) {
    if remap.map.is_empty() {
        return;
    }

    let map_len = remap.map.len();
    for (entity, mut visual) in visuals.iter_mut() {
        let old_index = visual.index;
        if old_index >= map_len {
            continue;
        }

        match remap.map[old_index] {
            Some(new_index) => visual.index = new_index,
            None => {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn clear_particle_remap(mut remap: ResMut<ParticleRemap>) {
    if !remap.map.is_empty() {
        remap.map.clear();
    }
}

fn log_particle_debug(state: Res<MpmState>, mut frame: Local<u32>) {
    const SAMPLE_PERIOD: u32 = 30;
    const SAMPLE_COUNT: usize = 3;

    if *frame % SAMPLE_PERIOD == 0 {
        let mut lines = Vec::new();
        let grid = state.grid();
        let particles = state.particles();

        for (idx, particle) in particles.iter().enumerate().take(SAMPLE_COUNT) {
            let interp = GridInterpolation::compute_for_particle(particle.position);
            let mut density = 0.0;
            for (coord, weight, _) in interp.iter_neighbors() {
                if let Some(cell) = grid.get_cell_coord(coord) {
                    density += cell.mass * weight;
                }
            }

            let speed = particle.velocity.length();
            let jacobian = particle.deformation_gradient.determinant();
            let volume = particle.mass * if density > 0.0 { 1.0 / density } else { 0.0 };

            lines.push(format!(
                "#{idx}: pos=({:.2},{:.2}) speed={:.2} dens={:.2} vol={:.2} J={:.2}",
                particle.position.x, particle.position.y, speed, density, volume, jacobian
            ));
        }

        if !lines.is_empty() {
            println!("[frame {:04}] {}", *frame, lines.join(" | "));
        }
    }

    *frame = frame.wrapping_add(1);
}

pub struct MpmPlugin;

impl Plugin for MpmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MpmState::new(SolverParams::default(), GRAVITY));
        app.insert_resource(ParticleRemap::default());
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.add_systems(Startup, (init_grid, init_particles).chain());
        app.add_systems(
            FixedUpdate,
            (
                zero_grid,
                particle_to_grid,
                cleanup_grid_cells,
                grid_update,
                log_particle_debug,
                grid_to_particle,
                remove_failed_particles_system,
                apply_particle_remap,
                clear_particle_remap,
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

#[derive(Component)]
struct DiagnosticsText;

fn setup_diagnostics(mut commands: Commands) {
    commands.spawn((
        Text::default(),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        DiagnosticsText,
    ));
}

fn update_diagnostics(
    diagnostics: Res<DiagnosticsStore>,
    state: Res<MpmState>,
    mut query: Query<&mut Text, With<DiagnosticsText>>,
) {
    let particle_count = state.particle_count();

    for mut text in &mut query {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);

        let frame_time = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|ft| ft.smoothed())
            .unwrap_or(0.0);

        text.0 = format!(
            "FPS: {:.1}\nFrame: {:.2}ms\nParticles: {}",
            fps, frame_time, particle_count
        );
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(MpmPlugin)
        .add_systems(Startup, (init, setup_diagnostics))
        .add_systems(Update, update_diagnostics)
        .run();
}



