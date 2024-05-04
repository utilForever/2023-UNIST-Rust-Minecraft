use crate::aabb::{get_block_aabb, AABB};
use crate::chunk_manager::ChunkManager;
use crate::constants::{
    HORIZONTAL_ACCELERATION, IN_AIR_FRICTION, JUMP_IMPULSE, MAX_VERTICAL_VELOCITY,
    MOUSE_SENSITIVITY_X, MOUSE_SENSITIVITY_Y, ON_GROUND_FRICTION, PLAYER_EYES_HEIGHT,
    PLAYER_HALF_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH, WALKING_SPEED,
};
use crate::input::InputCache;
use crate::util::Forward;
use glfw::Key;
use nalgebra::{clamp, Vector3};
use nalgebra_glm::{pi, vec2, vec3, Vec3};
use num_traits::Zero;
use std::ops::{Add, Mul};

pub struct PlayerProperties {
    pub rotation: Vec3,
}

impl PlayerProperties {
    pub fn new() -> Self {
        Self {
            rotation: vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn rotate_camera(&mut self, horizontal: f32, vertical: f32) {
        self.rotation.y += horizontal / 100.0 * MOUSE_SENSITIVITY_X;
        self.rotation.x += vertical / 100.0 * MOUSE_SENSITIVITY_Y;

        // Limit vertical movement
        self.rotation.x = clamp(
            self.rotation.x,
            -pi::<f32>() / 2.0 + 0.0001,
            pi::<f32>() / 2.0 - 0.0001,
        );
    }
}

#[derive(Clone)]
pub struct PlayerPhysicsState {
    pub position: Vec3,
    pub aabb: AABB,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub is_on_ground: bool,
}

impl PlayerPhysicsState {
    pub fn new_at_position(position: Vec3) -> Self {
        Self {
            position,
            aabb: {
                let mins = vec3(
                    position.x - PLAYER_HALF_WIDTH,
                    position.y,
                    position.z - PLAYER_HALF_WIDTH,
                );
                let maxs = vec3(
                    position.x + PLAYER_HALF_WIDTH,
                    position.y + PLAYER_HEIGHT,
                    position.z + PLAYER_HALF_WIDTH,
                );
                AABB::new(mins, maxs)
            },
            velocity: vec3(0.0, 0.0, 0.0),
            acceleration: vec3(0.0, 0.0, 0.0),
            is_on_ground: false,
        }
    }

    pub fn get_camera_position(&self) -> Vec3 {
        self.position + vec3(0.0, PLAYER_EYES_HEIGHT, 0.0)
    }
}

impl Mul<f32> for PlayerPhysicsState {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.position *= rhs;
        self.aabb.mins *= rhs;
        self.aabb.maxs *= rhs;
        self.velocity *= rhs;
        self.acceleration *= rhs;

        self
    }
}

impl Add for PlayerPhysicsState {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.position += rhs.position;
        self.aabb.mins += rhs.aabb.mins;
        self.aabb.maxs += rhs.aabb.maxs;
        self.velocity += rhs.velocity;
        self.acceleration += rhs.acceleration;

        self
    }
}

impl PlayerPhysicsState {
    pub fn apply_keyboard_movement(&mut self, rotation: &Vec3, input_cache: &InputCache) {
        // Jump
        if input_cache.is_key_pressed(Key::Space) {
            if self.is_on_ground {
                self.velocity.y = *JUMP_IMPULSE;
            }
        }

        // Walk
        let mut directional_acceleration = vec3(0.0, 0.0, 0.0);

        if input_cache.is_key_pressed(Key::W) {
            directional_acceleration +=
                -rotation.forward().cross(&Vector3::y()).cross(&Vector3::y());
        }

        if input_cache.is_key_pressed(Key::S) {
            directional_acceleration +=
                rotation.forward().cross(&Vector3::y()).cross(&Vector3::y());
        }

        if input_cache.is_key_pressed(Key::A) {
            directional_acceleration += -rotation.forward().cross(&Vector3::y())
        }

        if input_cache.is_key_pressed(Key::D) {
            directional_acceleration += rotation.forward().cross(&Vector3::y())
        }

        if directional_acceleration.norm_squared() != 0.0 {
            let directional_acceleration = directional_acceleration
                .normalize()
                .scale(HORIZONTAL_ACCELERATION);
            self.acceleration += directional_acceleration;
        }
    }

    pub fn get_colliding_block_coords(&self, chunk_manager: &ChunkManager) -> Option<Vec3> {
        let player_mins = &self.aabb.mins;
        let player_maxs = &self.aabb.maxs;

        let block_min = vec3(
            player_mins.x.floor() as i32,
            player_mins.y.floor() as i32,
            player_mins.z.floor() as i32,
        );
        let block_max = vec3(
            player_maxs.x.floor() as i32,
            player_maxs.y.floor() as i32,
            player_maxs.z.floor() as i32,
        );

        let mut block_collided = None;

        // Find the block that the player is colliding with
        'outer: for y in block_min.y..=block_max.y {
            for z in block_min.z..=block_max.z {
                for x in block_min.x..=block_max.x {
                    if let Some(block) = chunk_manager.get_block(x, y, z) {
                        if block.is_air() {
                            continue;
                        }

                        let block_aabb = get_block_aabb(&vec3(x as f32, y as f32, z as f32));

                        if self.aabb.intersects(&block_aabb) {
                            block_collided = Some(vec3(x as f32, y as f32, z as f32));
                            break 'outer;
                        }
                    }
                }
            }
        }

        block_collided
    }

    pub fn separate_from_block(&mut self, v: &Vec3, block_coords: &Vec3) -> bool {
        let mut is_player_on_ground = false;
        let block_aabb = get_block_aabb(&block_coords);

        if !v.x.is_zero() {
            if v.x < 0.0 {
                self.aabb = AABB::new(
                    vec3(block_aabb.maxs.x, self.aabb.mins.y, self.aabb.mins.z),
                    vec3(
                        block_aabb.maxs.x + PLAYER_WIDTH,
                        self.aabb.maxs.y,
                        self.aabb.maxs.z,
                    ),
                )
            } else {
                self.aabb = AABB::new(
                    vec3(
                        block_aabb.mins.x - PLAYER_WIDTH,
                        self.aabb.mins.y,
                        self.aabb.mins.z,
                    ),
                    vec3(block_aabb.mins.x, self.aabb.maxs.y, self.aabb.maxs.z),
                )
            }

            self.velocity.x = 0.0;
        }

        if !v.y.is_zero() {
            if v.y < 0.0 {
                self.aabb = AABB::new(
                    vec3(self.aabb.mins.x, block_aabb.maxs.y, self.aabb.mins.z),
                    vec3(
                        self.aabb.maxs.x,
                        block_aabb.maxs.y + PLAYER_HEIGHT,
                        self.aabb.maxs.z,
                    ),
                );
                is_player_on_ground = true;
            } else {
                self.aabb = AABB::new(
                    vec3(
                        self.aabb.mins.x,
                        block_aabb.mins.y - PLAYER_HEIGHT,
                        self.aabb.mins.z,
                    ),
                    vec3(self.aabb.maxs.x, block_aabb.mins.y, self.aabb.maxs.z),
                )
            }

            self.velocity.y = 0.0;
        }

        if !v.z.is_zero() {
            if v.z < 0.0 {
                self.aabb = AABB::new(
                    vec3(self.aabb.mins.x, self.aabb.mins.y, block_aabb.maxs.z),
                    vec3(
                        self.aabb.maxs.x,
                        self.aabb.maxs.y,
                        block_aabb.maxs.z + PLAYER_WIDTH,
                    ),
                )
            } else {
                self.aabb = AABB::new(
                    vec3(
                        self.aabb.mins.x,
                        self.aabb.mins.y,
                        block_aabb.mins.z - PLAYER_WIDTH,
                    ),
                    vec3(self.aabb.maxs.x, self.aabb.maxs.y, block_aabb.mins.z),
                )
            }

            self.velocity.z = 0.0;
        }

        is_player_on_ground
    }

    pub fn apply_friction(&mut self, dt: f32) {
        let friction = if self.is_on_ground {
            ON_GROUND_FRICTION
        } else {
            IN_AIR_FRICTION
        };

        if self.acceleration.x.is_zero() || self.acceleration.x.signum() != self.velocity.x.signum()
        {
            self.velocity.x -= friction * self.velocity.x * dt;
        }

        if self.acceleration.z.is_zero() || self.acceleration.z.signum() != self.velocity.z.signum()
        {
            self.velocity.z -= friction * self.velocity.z * dt;
        }
    }

    pub fn limit_velocity(&mut self) {
        // Limit the walking speed (horizontally)
        let mut horizontal = vec2(self.velocity.x, self.velocity.z);
        let speed = horizontal.magnitude();

        if speed > WALKING_SPEED {
            horizontal = horizontal.scale(WALKING_SPEED / speed);
        }

        self.velocity.x = horizontal.x;
        self.velocity.z = horizontal.y;

        // Limit the free-falling speed (vertically)
        // NOTE: https://www.planetminecraft.com/blog/the-acceleration-of-gravity-in-minecraft-and-terminal-velocity/
        if self.velocity.y < -MAX_VERTICAL_VELOCITY {
            self.velocity.y = -MAX_VERTICAL_VELOCITY;
        }
    }
}
