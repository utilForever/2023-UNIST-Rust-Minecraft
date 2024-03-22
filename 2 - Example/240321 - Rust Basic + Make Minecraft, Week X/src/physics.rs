use crate::aabb::AABB;
use crate::{PLAYER_EYES_HEIGHT, PLAYER_HALF_WIDTH, PLAYER_HEIGHT};
use nalgebra_glm::{vec3, Vec3};
use std::ops::{Add, Mul};
use std::time;

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

pub struct PhysicsManager {
    pub t: f32,
    pub dt: f32,
    pub current_time: time::Instant,
    pub accumulator: f32,
    pub previous_state: PlayerPhysicsState,
    pub current_state: PlayerPhysicsState,
}

impl PhysicsManager {
    pub fn new(dt: f32, initial_state: PlayerPhysicsState) -> Self {
        Self {
            t: 0.0,
            dt,
            current_time: time::Instant::now(),
            accumulator: 0.0,
            previous_state: initial_state.clone(),
            current_state: initial_state,
        }
    }

    pub fn step(
        &mut self,
        integrate: &dyn Fn(PlayerPhysicsState, f32, f32) -> PlayerPhysicsState,
    ) -> PlayerPhysicsState {
        let now = time::Instant::now();
        let mut frame_time = now.duration_since(self.current_time).as_secs_f32();

        if frame_time > 0.25 {
            frame_time = 0.25;
        }

        self.current_time = now;
        self.accumulator += frame_time;

        while self.accumulator >= self.dt {
            self.previous_state = self.current_state.clone();
            self.current_state = integrate(self.previous_state.clone(), self.t, self.dt);
            self.t += self.dt;
            self.accumulator -= self.dt;
        }

        let alpha = self.accumulator / self.dt;
        let state =
            self.current_state.clone() * alpha + self.previous_state.clone() * (1.0 - alpha);

        state
    }

    pub fn get_current_state(&mut self) -> &mut PlayerPhysicsState {
        &mut self.current_state
    }
}

pub fn get_block_aabb(mins: &Vec3) -> AABB {
    AABB::new(mins.clone(), mins + vec3(1.0, 1.0, 1.0))
}
