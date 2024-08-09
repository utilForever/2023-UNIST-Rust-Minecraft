use crate::constants::{
    FLYING_TRIGGER_INTERVAL, FOV, JUMP_IMPULSE, PLAYER_EYES_HEIGHT, SPRINTING_TRIGGER_INTERVAL,
};
use crate::input::InputCache;
use crate::physics::Interpolator;
use crate::player::{PlayerPhysicsState, PlayerState};
use crate::timer::Timer;
use specs::{Join, Read, ReadStorage, System, WriteStorage};
use std::time::Instant;

pub struct HandlePlayerInput;

impl<'a> System<'a> for HandlePlayerInput {
    type SystemData = (
        Read<'a, InputCache>,
        WriteStorage<'a, PlayerState>,
        WriteStorage<'a, Interpolator<PlayerPhysicsState>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input_cache, mut player_state, mut player_physics_state) = data;

        for (player_state, player_physics_state) in
            (&mut player_state, &mut player_physics_state).join()
        {
            let mut player_physics_state = player_physics_state.get_latest_state_mut();

            for event in &input_cache.events {
                match event {
                    glfw::WindowEvent::CursorPos(_, _) => {
                        player_state.rotate_camera(
                            input_cache.cursor_rel_pos.x as f32,
                            input_cache.cursor_rel_pos.y as f32,
                        );
                    }
                    glfw::WindowEvent::Key(glfw::Key::Space, _, glfw::Action::Press, _) => {
                        // Player state
                        if player_state.fly_throttle {
                            player_state.fly_throttle = false;
                        } else if Instant::now().duration_since(player_state.fly_last_toggled)
                            < *FLYING_TRIGGER_INTERVAL
                        {
                            player_state.is_flying = !player_state.is_flying;
                            println!("Flying: {}", player_state.is_flying);
                            player_state.fly_throttle = true;
                        }

                        player_state.fly_last_toggled = Instant::now();

                        // Player physics state
                        if player_physics_state.is_on_ground {
                            player_physics_state.velocity.y = *JUMP_IMPULSE;
                            player_state.jump_last_executed = Instant::now();
                        }
                    }
                    glfw::WindowEvent::Key(glfw::Key::LeftShift, _, glfw::Action::Release, _) => {
                        player_state.is_sneaking = false;
                    }
                    glfw::WindowEvent::Key(glfw::Key::W, _, glfw::Action::Press, _) => {
                        if player_state.sprint_throttle {
                            player_state.sprint_throttle = false;
                        } else if Instant::now().duration_since(player_state.sprint_last_toggled)
                            < *SPRINTING_TRIGGER_INTERVAL
                        {
                            player_state.is_sprinting = true;
                            player_state.sprint_throttle = true;
                        }

                        player_state.sprint_last_toggled = Instant::now();
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct UpdatePlayerState;

impl<'a> System<'a> for UpdatePlayerState {
    type SystemData = (
        Read<'a, Timer>,
        Read<'a, InputCache>,
        WriteStorage<'a, PlayerState>,
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (global_timer, input_cache, mut player_state, mut player_physics_state) = data;

        for (player_state, player_physics_state) in
            (&mut player_state, &player_physics_state).join()
        {
            let mut player_physics_state = player_physics_state.get_latest_state();
            let t = global_timer.time();

            // Movement
            if input_cache.is_key_pressed(glfw::Key::LeftShift) && player_physics_state.is_on_ground
            {
                player_state.is_sneaking = true;
                player_state.is_sprinting = false;
            }

            if input_cache.is_key_pressed(glfw::Key::LeftControl)
                && input_cache.is_key_pressed(glfw::Key::W)
                && !player_state.is_sneaking
            {
                player_state.is_sprinting = true;
            }

            if player_state.is_sprinting && !input_cache.is_key_pressed(glfw::Key::W) {
                player_state.is_sprinting = false;
            }

            // Camera height
            let target_camera_height = if player_state.is_sneaking {
                PLAYER_EYES_HEIGHT - 1.0 / 8.0
            } else {
                PLAYER_EYES_HEIGHT
            };

            player_state
                .camera_height
                .interpolate_camera_height(t, target_camera_height);

            // Field of view
            let target_fov = if player_state.is_flying {
                if player_state.is_sprinting {
                    FOV + FOV * 0.30
                } else {
                    FOV + FOV * 0.15
                }
            } else {
                if player_state.is_sprinting {
                    FOV + FOV * 0.15
                } else {
                    FOV
                }
            };

            player_state.fov.interpolate_fov(t, target_fov);
        }
    }
}
