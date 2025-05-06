use bevy::prelude::*;

use crate::constants;

pub const GRID_RESOLUTION: usize = 128;
const GRAVITY: Vec2 = Vec2::new(0.0, -80.0);

const EOS_STIFFNESS: f32 = 10.0;
const EOS_POWER: u8 = 4;

const REST_DENSITY: f32 = 2.0;
const DYNAMIC_VISCOSITY: f32 = 0.1;

#[derive(Component)]
pub struct Cell {
    pub velocity: Vec2,
    pub mass: f32,
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

#[derive(Resource, Default)]
pub struct Grid {
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn zero_all_cells(&mut self) {
        self.cells.iter_mut().for_each(|cell| cell.zero());
    }
    
    pub fn get_cell_mut(&mut self, pos: UVec2) -> Option<&mut Cell> {
        let index = get_cell_index(pos);
        self.cells.get_mut(index)
    }
    
    pub fn get_cell(&self, pos: UVec2) -> Option<&Cell> {
        let index = get_cell_index(pos);
        self.cells.get(index)
    }
}

#[derive(Component)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub affine_momentum_matrix: Mat2,
    pub material_type: MaterialType,
}

impl Particle {
    pub fn zeroed(material_type: MaterialType) -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0,
            affine_momentum_matrix: Mat2::ZERO,
            material_type,
        }
    }
}

pub enum MaterialType {
    Water { vp0: f32, ap: f32, jp: f32 },
}

impl MaterialType {
    fn constitutive_model(&mut self) {
        match self {
            Self::Water { vp0, ap, jp } => {
                let djp = -constants::k_water * (1.0 / jp.powf(constants::gamma_water) - 1.0);
                *ap = djp * *vp0 * *jp;
            }
        }
    }

    fn update_deformation(&mut self, t: Mat2, dt: f32) {
        match self {
            Self::Water { vp0: _, ap: _, jp } => {
                *jp = (1.0 + dt * (t.col(0).x + t.col(1).y)) * *jp;
            }
        }
    }
}

#[inline]
fn get_cell_index(pos: UVec2) -> usize {
    pos.y as usize * GRID_RESOLUTION + pos.x as usize
}

// Helper function to calculate grid weights and positions
fn calculate_grid_weights(particle_position: Vec2) -> (UVec2, [Vec2; 3]) {
    let cell_index = particle_position.as_uvec2();
    let cell_difference = (particle_position - cell_index.as_vec2()) - 0.5;
    
    // Calculate weights for each dimension separately
    let x_weights = [
        0.5 * (0.5 - cell_difference.x) * (0.5 - cell_difference.x),
        0.75 - cell_difference.x * cell_difference.x,
        0.5 * (0.5 + cell_difference.x) * (0.5 + cell_difference.x)
    ];
    
    let y_weights = [
        0.5 * (0.5 - cell_difference.y) * (0.5 - cell_difference.y),
        0.75 - cell_difference.y * cell_difference.y,
        0.5 * (0.5 + cell_difference.y) * (0.5 + cell_difference.y)
    ];
    
    // Combine into Vec2 array
    let weights = [
        Vec2::new(x_weights[0], y_weights[0]),
        Vec2::new(x_weights[1], y_weights[1]),
        Vec2::new(x_weights[2], y_weights[2])
    ];
    
    (cell_index, weights)
}

pub fn zero_grid(mut grid: ResMut<Grid>) {
    grid.zero_all_cells();
}

pub fn particle_to_grid_1(
    query: Query<(Entity, &Particle)>,
    mut grid: ResMut<Grid>
) {
    for (_, particle) in query.iter() {
        let (cell_index, weights) = calculate_grid_weights(particle.position);

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                let cell_distance =
                    (cell_position.as_vec2() - particle.position) + 0.5;
                let q = particle.affine_momentum_matrix * cell_distance;

                let mass_contribution = weight * particle.mass;

                // Use the helper function for cell indexing
                let cell_index = get_cell_index(cell_position);

                let cell = grid.cells.get_mut(cell_index).unwrap();

                cell.mass += mass_contribution;

                cell.velocity += mass_contribution * (particle.velocity + q);
            }
        }
    }
}

pub fn particle_to_grid_2(
    time: Res<Time>,
    query: Query<&Particle>,
    mut grid: ResMut<Grid>
) {
    for particle in query {
        let (cell_index, weights) = calculate_grid_weights(particle.position);

        let mut density = 0.0;

        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;

                let cell_position =
                    UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);

                // Use the helper function for cell indexing
                let cell_index = get_cell_index(cell_position);

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

                // Use the helper function for cell indexing
                let cell_index = get_cell_index(cell_position);
                let cell = grid.cells.get_mut(cell_index).unwrap();

                let momentum = eq_16_term_0 * weight * cell_distance;

                cell.velocity += momentum;
            }
        }
    }
}

pub fn calculate_grid_velocities(
    time: Res<Time>,
    mut grid: ResMut<Grid>
) {
    for (index, cell) in grid.cells.iter_mut().enumerate() {
        if cell.mass > 0.0 {
            let gravity_velocity = time.delta_secs() * GRAVITY;
            cell.velocity /= cell.mass;
            cell.velocity += gravity_velocity;

            // Fixed indexing: index = y * width + x
            // So y = index / width, x = index % width
            let y = index / GRID_RESOLUTION;
            let x = index % GRID_RESOLUTION;

            if x < 2 || x > GRID_RESOLUTION - 3 {
                cell.velocity.x = 0.0;
            }

            if y < 2 || y > GRID_RESOLUTION - 3 {
                cell.velocity.y = 0.0;
            }
        }
    }
}

pub fn grid_to_particle(
    time: Res<Time>,
    mut query: Query<&mut Particle>,
    grid: Res<Grid>
) {
    query.par_iter_mut()
        .for_each(|mut particle| {
            particle.velocity = Vec2::ZERO;

            let (cell_index, weights) = calculate_grid_weights(particle.position);

            let mut b = Mat2::ZERO;

            for gx in 0..3 {
                for gy in 0..3 {
                    let weight = weights[gx].x * weights[gy].y;

                    let cell_position =
                        UVec2::new(cell_index.x + gx as u32 - 1, cell_index.y + gy as u32 - 1);
                    
                    // Use the helper function for cell indexing
                    let cell_index = get_cell_index(cell_position);

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
        });
}