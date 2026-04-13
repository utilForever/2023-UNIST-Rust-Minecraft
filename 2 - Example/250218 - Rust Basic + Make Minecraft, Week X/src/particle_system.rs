use crate::aabb::get_block_aabb;
use crate::chunk::BlockID;
use crate::chunk_manager::ChunkManager;
use crate::physics::{Interpolatable, Interpolator};
use crate::shader::ShaderProgram;
use crate::shapes::quad_array_texture;
use crate::types::TexturePack;
use itertools::Itertools;
use nalgebra::Matrix4;
use nalgebra_glm::{vec3, vec4, Mat4, Vec3};
use num_traits::Zero;
use rand::random;
use std::ffi::c_void;
use std::ptr::null;
use std::time::{Duration, Instant};

pub struct ParticleSystem {
    particles: Vec<Particle>,
    index_available: usize,
    last_updated: Instant,
    vao: u32,
    vbo: u32,
}

impl ParticleSystem {
    pub fn new(max_instances: usize) -> ParticleSystem {
        // Allocate VRAM for `max_instances` particles

        // Setup VAO
        let mut vao = 0;

        gl_call!(gl::CreateVertexArrays(1, &mut vao));

        // Position
        gl_call!(gl::EnableVertexArrayAttrib(vao, 0));
        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            0,
            4,
            gl::FLOAT,
            gl::FALSE,
            0
        ));
        gl_call!(gl::VertexArrayAttribBinding(vao, 0, 0));

        // Texture coords
        gl_call!(gl::EnableVertexArrayAttrib(vao, 1));
        gl_call!(gl::VertexArrayAttribFormat(
            vao,
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            4 * std::mem::size_of::<f32>() as u32
        ));
        gl_call!(gl::VertexArrayAttribBinding(vao, 1, 0));

        // Setup VBO
        // Position and texture coords interleaved
        let mut vbo = 0;

        gl_call!(gl::CreateBuffers(1, &mut vbo));
        gl_call!(gl::NamedBufferData(
            vbo,
            (max_instances * 6 * 7 * std::mem::size_of::<f32>()) as isize,
            null(),
            gl::DYNAMIC_DRAW
        ));
        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            0,
            vbo,
            0,
            (7 * std::mem::size_of::<f32>()) as i32
        ));

        ParticleSystem {
            particles: {
                let mut vec = Vec::new();
                vec.resize_with(max_instances, Particle::default);
                vec
            },
            index_available: max_instances - 1,
            last_updated: Instant::now(),
            vao,
            vbo,
        }
    }

    pub fn emit(&mut self, particle_props: &ParticleProps, uv_map: &TexturePack, block: BlockID) {
        let get_texture_coords = |uv: (f32, f32, f32, f32), layer: f32| {
            (&[
                uv.0, uv.1, layer, uv.2, uv.1, layer, uv.2, uv.3, layer, uv.2, uv.3, layer, uv.0,
                uv.3, layer, uv.0, uv.1, layer,
            ])
                .to_vec()
        };

        self.particles[self.index_available] = Particle {
            active: true,
            physics_properties: Interpolator::new(
                1.0 / 30.0,
                ParticlePhysicsProperties {
                    position: particle_props.position,
                    velocity: particle_props.velocity,
                    acceleration: particle_props.acceleration,
                },
            ),
            tex_coords: {
                let uv_x = random::<f32>();
                let uv_y = random::<f32>();
                let uv = (uv_x, uv_y, uv_x + 0.2, uv_y + 0.2);

                get_texture_coords(
                    uv,
                    uv_map.get(&block).unwrap().get_uv_of_every_face().0 as f32,
                )
            },
            scale: particle_props.scale,
            life_remaining: particle_props.lifetime,
        };

        self.index_available = if self.index_available == 0 {
            self.particles.len() - 1
        } else {
            self.index_available - 1
        };
    }

    pub fn update_all_particles(&mut self, time: Instant, chunk_manager: &ChunkManager) {
        let time_passed = time.saturating_duration_since(self.last_updated);
        self.last_updated = time;

        for particle in &mut self.particles.iter_mut().filter(|p| p.active) {
            if let Some(life_remaining) = particle.life_remaining.checked_sub(time_passed) {
                particle.life_remaining = life_remaining;
            } else {
                particle.active = false;
                continue;
            }

            particle
                .physics_properties
                .update_particle(time, chunk_manager);
        }
    }

    pub fn render_all_particles(
        &mut self,
        _shader: &mut ShaderProgram,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
    ) {
        let mut vbo_data: Vec<f32> = Vec::new();

        // Prepare the VBOs
        let mut active_particles = 0;

        for particle in self.particles.iter().filter(|p| p.active) {
            active_particles += 1;

            let state = particle.physics_properties.get_interpolated_state();
            let model_matrix = {
                let translate_matrix = Matrix4::new_translation(&state.position);
                let rotate_matrix = Matrix4::from_euler_angles(0.0f32, 0.0, 0.0);

                translate_matrix * rotate_matrix
            };

            let mut model_view = view_matrix * model_matrix;

            model_view.m11 = particle.scale.x;
            model_view.m12 = 0.0;
            model_view.m13 = 0.0;

            model_view.m21 = 0.0;
            model_view.m22 = particle.scale.y;
            model_view.m23 = 0.0;

            model_view.m31 = 0.0;
            model_view.m32 = 0.0;
            model_view.m33 = particle.scale.z;

            let mvp = projection_matrix * model_view;
            let quad = quad_array_texture();
            let pos_chunks = quad.iter().chunks(3);
            let tex_chunks = particle.tex_coords.iter().chunks(3);
            let quad_vertices = pos_chunks.into_iter().zip(&tex_chunks);

            for (mut pos, tex) in quad_vertices {
                let pos = vec4(
                    *pos.next().unwrap(),
                    *pos.next().unwrap(),
                    *pos.next().unwrap(),
                    1.0,
                );
                vbo_data.extend(&(mvp * pos));
                vbo_data.extend(tex);
            }
        }

        gl_call!(gl::NamedBufferSubData(
            self.vbo,
            0,
            (vbo_data.len() * std::mem::size_of::<f32>()) as isize,
            vbo_data.as_ptr() as *mut c_void
        ));
        gl_call!(gl::BindVertexArray(self.vao));
        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6 * active_particles));
    }
}

#[derive(Clone)]
pub struct ParticlePhysicsProperties {
    pub position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
}

impl Default for ParticlePhysicsProperties {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            velocity: Vec3::zero(),
            acceleration: Vec3::zero(),
        }
    }
}

impl Interpolatable for ParticlePhysicsProperties {
    fn interpolate(&self, other: &Self, alpha: f32) -> Self {
        let interpolate_vec3 = |from: &Vec3, to: &Vec3| alpha * from + (1.0 - alpha) * to;

        ParticlePhysicsProperties {
            position: interpolate_vec3(&self.position, &other.position),
            velocity: interpolate_vec3(&self.velocity, &other.velocity),
            acceleration: interpolate_vec3(&self.acceleration, &other.acceleration),
        }
    }
}

impl Interpolator<ParticlePhysicsProperties> {
    fn update_particle(&mut self, time: Instant, chunk_manager: &ChunkManager) {
        self.step(time, &mut |state, _t, dt| {
            let mut state = state.clone();
            state.velocity += state.acceleration * dt;

            let vectors = &[
                vec3(state.velocity.x, 0.0, 0.0),
                vec3(0.0, state.velocity.y, 0.0),
                vec3(0.0, 0.0, state.velocity.z),
            ];

            for vector in vectors {
                state.position += vector * dt;

                let containing_block = vec3(
                    state.position.x.floor() as i32,
                    state.position.y.floor() as i32,
                    state.position.z.floor() as i32,
                );
                let mut colliding_block_aabb = None;

                if let Some(block) = chunk_manager.get_block(
                    containing_block.x,
                    containing_block.y,
                    containing_block.z,
                ) {
                    if block.is_air() {
                        continue;
                    }

                    let block_aabb = get_block_aabb(&vec3(
                        containing_block.x as f32,
                        containing_block.y as f32,
                        containing_block.z as f32,
                    ));
                    colliding_block_aabb = Some(block_aabb);
                }

                if colliding_block_aabb.is_none() {
                    continue;
                }

                let colliding_block_aabb = colliding_block_aabb.unwrap();
                let padding = 0.001;

                if !vector.x.is_zero() {
                    if vector.x < 0.0 {
                        state.position.x = colliding_block_aabb.maxs.x + padding;
                    } else {
                        state.position.x = colliding_block_aabb.mins.x - padding;
                    }

                    state.velocity.x *= -0.1;
                }

                if !vector.y.is_zero() {
                    if vector.y < 0.0 {
                        state.position.y = colliding_block_aabb.maxs.y + padding;
                    } else {
                        state.position.y = colliding_block_aabb.mins.y - padding;
                    }

                    state.velocity.y *= -0.1;
                }

                if !vector.z.is_zero() {
                    if vector.z < 0.0 {
                        state.position.z = colliding_block_aabb.maxs.z + padding;
                    } else {
                        state.position.z = colliding_block_aabb.mins.z - padding;
                    }

                    state.velocity.z *= -0.1;
                }
            }

            state.velocity.x *= 0.8;
            state.velocity.z *= 0.8;

            state
        });
    }
}

pub struct ParticleProps {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub lifetime: Duration,
    pub scale: Vec3,
}

struct Particle {
    active: bool,
    physics_properties: Interpolator<ParticlePhysicsProperties>,
    tex_coords: Vec<f32>,
    scale: Vec3,
    life_remaining: Duration,
}

impl Default for Particle {
    fn default() -> Self {
        Particle {
            active: false,
            physics_properties: Interpolator::default(),
            tex_coords: Vec::default(),
            scale: Vec3::zero(),
            life_remaining: Duration::default(),
        }
    }
}

impl ParticleSystem {
    pub fn spawn_block_breaking_particles(
        &mut self,
        pos: Vec3,
        uv_map: &TexturePack,
        block: BlockID,
    ) {
        let block_center = pos + vec3(0.5, 0.5, 0.5);
        let half_spacing = 1.0 / 8.0;

        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    let particle_pos = pos
                        + vec3(half_spacing, half_spacing, half_spacing)
                        + vec3(
                            x as f32 * 2.0 * half_spacing,
                            y as f32 * 2.0 * half_spacing,
                            z as f32 * 2.0 * half_spacing,
                        );

                    self.emit(
                        &ParticleProps {
                            position: particle_pos,
                            velocity: {
                                let from_center = particle_pos - block_center;
                                let vx = from_center.x * 5.0 + 4.0 * random::<f32>() - 2.0;
                                let vy = from_center.y * 10.0 + 4.0 * random::<f32>() - 2.0;
                                let vz = from_center.z * 5.0 + 4.0 * random::<f32>() - 2.0;

                                vec3(vx, vy, vz)
                            },
                            acceleration: vec3(0.0, -30.0, 0.0),
                            lifetime: Duration::from_millis(100 + random::<u64>() % 750),
                            scale: {
                                let size = 0.1 + random::<f32>() * 1.5 / 10.0;
                                Vec3::new(size, size, size)
                            },
                        },
                        &uv_map,
                        block,
                    );
                }
            }
        }
    }
}
