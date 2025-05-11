use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;
use mpm2d::simulation::MaterialType;
use mpm2d::solver::prelude::*;
use mpm2d::PbmpmPlugin;

fn init_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rand = rand::rng();
    for x in 0..50 {
        for y in 0..100 {
            commands.spawn((
                Particle {
                    position: Vec2 {
                        x: 16.0 + x as f32 / 4.0,
                        y: 32.0 + y as f32 / 4.0,
                    },
                    velocity: Vec2::new(rand.random_range(-10.0..=10.0), rand.random_range(-10.0..=10.0)),
                    mass: 1.0,
                    affine_momentum_matrix: Mat2::ZERO,
                    deformation_displacement: Mat2::ZERO,
                    prev_deformation_displacement: Mat2::ZERO,
                    liquid_density: 1.0,
                    material_type: MaterialType::Liquid { vp0: 1.0, ap: 0.0, jp: 1.0 },
                },
                Mesh2d(meshes.add(Circle::new(1.0))),
                MeshMaterial2d(materials.add(Color::hsl(210.0, 0.7, 0.3))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
    for x in 0..50 {
        for y in 0..100 {
            commands.spawn((
                Particle {
                    position: Vec2 {
                        x: 112.0 + x as f32 / 4.0,
                        y: 32.0 + y as f32 / 4.0,
                    },
                    velocity: Vec2::new(rand.random_range(-10.0..=10.0), rand.random_range(-10.0..=10.0)),
                    mass: 1.0,
                    affine_momentum_matrix: Mat2::ZERO,
                    deformation_displacement: Mat2::ZERO,
                    prev_deformation_displacement: Mat2::ZERO,
                    liquid_density: 1.0,
                    material_type: MaterialType::Liquid { vp0: 1.0, ap: 0.0, jp: 1.0 },
                },
                Mesh2d(meshes.add(Circle::new(1.0))),
                MeshMaterial2d(materials.add(Color::hsl(210.0, 0.7, 0.3))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
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
) {
    let Ok(_window) = window.single() else {
        return;
    };
    let Ok((_camera, mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let fspeed = 600.0 * time.delta_secs();

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

    if mouse.pressed(MouseButton::Left) {
        let mut rand = rand::rng();
        let handle = meshes.add(Circle::new(1.0));

        commands.spawn((
            Particle {
                position: Vec2 {
                    x: 64.0,
                    y: 64.0,
                },
                velocity: Vec2::new(rand.random_range(-10.0..=10.0), rand.random_range(-50.0..=-20.0)),
                mass: 1.0,
                affine_momentum_matrix: Mat2::ZERO,
                deformation_displacement: Mat2::ZERO,
                prev_deformation_displacement: Mat2::ZERO,
                liquid_density: 1.0,
                material_type: MaterialType::Liquid { vp0: 1.0, ap: 0.0, jp: 1.0 },
            },
            Mesh2d(handle),
            MeshMaterial2d(materials.add(Color::hsl(0.0, 1.0, 0.5))),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
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

fn update_particle_transforms(
    mut query: Query<(&mut Transform, &Particle)>,
) {
    query.par_iter_mut().for_each(|(mut transform, particle)| {
        transform.translation = Vec3::new(
            (particle.position.x - 64.0) * 4.0, 
            (particle.position.y - 64.0) * 4.0, 
            0.0
        );
    });
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PbmpmPlugin::default()) // Use the plugin from our library
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 60.0)))
        .add_systems(Startup, (init, init_particles))
        .add_systems(FixedUpdate, (update_particle_transforms, controls).chain())
        .run();
}