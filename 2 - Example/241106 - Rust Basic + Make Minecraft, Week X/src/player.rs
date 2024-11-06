use crate::aabb::{get_block_aabb, AABB};
use crate::chunk_manager::ChunkManager;
use crate::constants::{
    FLYING_SPEED, FLYING_SPRINTING_SPEED, FOV, HORIZONTAL_ACCELERATION, IN_AIR_FRICTION,
    JUMP_IMPULSE, MAX_VERTICAL_VELOCITY, MOUSE_SENSITIVITY_X, MOUSE_SENSITIVITY_Y,
    ON_GROUND_FRICTION, PLAYER_EYES_HEIGHT, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH,
    SNEAKING_SPEED, SPRINTING_SPEED, WALKING_SPEED,
};
use crate::input::InputCache;
use crate::physics::{Interpolatable, Interpolator};
use crate::util::Forward;
use nalgebra::{clamp, Vector3};
use nalgebra_glm::{pi, vec2, vec3, IVec3, Mat4, Vec3};
use num_traits::Zero;
use std::time::Instant;

pub struct PlayerState {
    pub rotation: Vec3,
    pub camera_height: Interpolator<f32>,
    pub fov: Interpolator<f32>,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,

    pub is_on_ground: bool,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_flying: bool,

    pub targeted_block: Option<((i32, i32, i32), IVec3)>,

    pub(crate) jump_last_executed: Instant,
    pub(crate) fly_throttle: bool,
    pub(crate) fly_last_toggled: Instant,
    pub(crate) sprint_throttle: bool,
    pub(crate) sprint_last_toggled: Instant,
    pub(crate) block_placing_last_executed: Instant,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            rotation: vec3(0.0, 0.0, 0.0),
            camera_height: Interpolator::new(1.0 / 30.0, PLAYER_EYES_HEIGHT),
            fov: Interpolator::new(1.0 / 30.0, FOV),
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),

            is_on_ground: false,
            is_sneaking: false,
            is_sprinting: false,
            is_flying: false,

            targeted_block: None,

            jump_last_executed: Instant::now(),
            fly_throttle: false,
            fly_last_toggled: Instant::now(),
            sprint_throttle: false,
            sprint_last_toggled: Instant::now(),
            block_placing_last_executed: Instant::now(),
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
        }
    }
}

impl Interpolatable for PlayerPhysicsState {
    fn interpolate(&self, other: &Self, alpha: f32) -> Self {
        let interpolate_vec3 = |from: &Vec3, to: &Vec3| alpha * from + (1.0 - alpha) * to;

        Self {
            position: interpolate_vec3(&self.position, &other.position),
            aabb: AABB {
                mins: interpolate_vec3(&self.aabb.mins, &other.aabb.mins),
                maxs: interpolate_vec3(&self.aabb.maxs, &other.aabb.maxs),
            },
            velocity: interpolate_vec3(&self.velocity, &other.velocity),
            acceleration: interpolate_vec3(&self.acceleration, &other.acceleration),
        }
    }
}

impl PlayerPhysicsState {
    pub fn apply_keyboard_movement(
        &mut self,
        player_properties: &mut PlayerState,
        input_cache: &InputCache,
    ) {
        let rotation = &player_properties.rotation;

        // Flying
        if player_properties.is_flying {
            if input_cache.is_key_pressed(glfw::Key::Space) {
                self.acceleration = vec3(0.0, 100.0, 0.0);
            }

            if input_cache.is_key_pressed(glfw::Key::LeftShift) {
                self.acceleration = vec3(0.0, -100.0, 0.0);
            }
        }

        if input_cache.is_key_pressed(glfw::Key::Space) {
            let now = Instant::now();

            if now
                .duration_since(player_properties.jump_last_executed)
                .as_secs_f32()
                >= 0.475
            {
                if player_properties.is_on_ground {
                    self.velocity.y = *JUMP_IMPULSE;
                    player_properties.jump_last_executed = now;
                }
            }
        }

        // Walk
        let mut directional_acceleration = vec3(0.0, 0.0, 0.0);

        if input_cache.is_key_pressed(glfw::Key::W) {
            directional_acceleration +=
                -rotation.forward().cross(&Vector3::y()).cross(&Vector3::y());
        }

        if input_cache.is_key_pressed(glfw::Key::S) {
            directional_acceleration +=
                rotation.forward().cross(&Vector3::y()).cross(&Vector3::y());
        }

        if input_cache.is_key_pressed(glfw::Key::A) {
            directional_acceleration += -rotation.forward().cross(&Vector3::y())
        }

        if input_cache.is_key_pressed(glfw::Key::D) {
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

    pub fn apply_friction(&mut self, dt: f32, player_state: &PlayerState) {
        let friction = if player_state.is_on_ground {
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

        if player_state.is_flying {
            if self.acceleration.y.is_zero()
                || self.acceleration.y.signum() != self.velocity.y.signum()
            {
                self.velocity.y -= ON_GROUND_FRICTION * self.velocity.y * dt;
            }
        }
    }

    pub fn limit_velocity(&mut self, player_properties: &PlayerState) {
        // Limit the horizontal speed
        let mut horizontal = vec2(self.velocity.x, self.velocity.z);
        let speed = horizontal.magnitude();

        let max_speed = if player_properties.is_flying {
            self.velocity.y = clamp(self.velocity.y, -8.0, 8.0);

            if player_properties.is_sprinting {
                FLYING_SPRINTING_SPEED
            } else {
                FLYING_SPEED
            }
        } else {
            if player_properties.is_sneaking {
                SNEAKING_SPEED
            } else if player_properties.is_sprinting {
                SPRINTING_SPEED
            } else {
                WALKING_SPEED
            }
        };

        if speed > max_speed {
            horizontal = horizontal.scale(max_speed / speed);
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
