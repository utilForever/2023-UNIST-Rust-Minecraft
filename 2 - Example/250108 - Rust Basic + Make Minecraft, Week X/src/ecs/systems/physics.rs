use crate::chunk_manager::ChunkManager;
use crate::constants::{GRAVITY, PLAYER_HALF_WIDTH};
use crate::input::InputCache;
use crate::physics::Interpolator;
use crate::player::{PlayerPhysicsState, PlayerState};
use crate::timer::Timer;
use nalgebra_glm::vec3;
use num_traits::Zero;
use specs::{Join, Read, System, WriteStorage};
use std::sync::Arc;

pub struct UpdatePlayerPhysics;

impl<'a> System<'a> for UpdatePlayerPhysics {
    type SystemData = (
        Read<'a, Timer>,
        Read<'a, InputCache>,
        Read<'a, Arc<ChunkManager>>,
        WriteStorage<'a, Interpolator<PlayerPhysicsState>>,
        WriteStorage<'a, PlayerState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (global_timer, input_cache, chunk_manager, mut player_physics_state, mut player_state) =
            data;

        for (player_physics_state, player_state) in
            (&mut player_physics_state, &mut player_state).join()
        {
            player_physics_state.step(
                global_timer.time(),
                &mut |player: &PlayerPhysicsState, _t: f32, dt: f32| {
                    let mut player = player.clone();

                    if !player_state.is_flying {
                        player.acceleration.y += GRAVITY;
                    }

                    player.apply_keyboard_movement(player_state, &input_cache);
                    player.velocity += player.acceleration * dt;
                    player.apply_friction(dt, &player_state);
                    player.limit_velocity(&player_state);

                    let will_hit_ground = |player: &PlayerPhysicsState| {
                        let mut player = player.clone();
                        let v_y = vec3(0.0, player.velocity.y, 0.0);

                        player.aabb.translate(&(v_y * dt));

                        let colliding_block = player.get_colliding_block_coords(&chunk_manager);

                        if let Some(colliding_block) = colliding_block {
                            player.separate_from_block(&v_y, &colliding_block)
                        } else {
                            false
                        }
                    };

                    // We are using the Separated Axis Theorem
                    // We decompose the velocity vector into 3 vectors for each dimension
                    // For each one, we move the entity and do the collision detection/resolution
                    let mut is_player_on_ground = false;

                    let separated_axis = &[
                        vec3(player.velocity.x, 0.0, 0.0),
                        vec3(0.0, 0.0, player.velocity.z),
                        vec3(0.0, player.velocity.y, 0.0),
                    ];

                    for v in separated_axis {
                        let backup = player.clone();

                        player.aabb.translate(&(v * dt));
                        let block_collided = player.get_colliding_block_coords(&chunk_manager);

                        // Collision resolution
                        if let Some(block_collided) = block_collided {
                            is_player_on_ground |= player.separate_from_block(&v, &block_collided);
                        }

                        // If the player is sneaking and is not on the ground, the player should not be able to move
                        if input_cache.is_key_pressed(glfw::Key::LeftShift)
                            && player_state.is_on_ground
                            && !will_hit_ground(&player)
                            && player.velocity.y < 0.0
                        {
                            player = backup;

                            if !v.x.is_zero() {
                                player.velocity.x = 0.0;
                            }

                            if !v.z.is_zero() {
                                player.velocity.z = 0.0;
                            }
                        }
                    }

                    player_state.is_on_ground = is_player_on_ground;

                    if player_state.is_on_ground {
                        player_state.is_flying = false;
                    }

                    // Update the position of the player and reset the acceleration
                    player.position.x = player.aabb.mins.x + PLAYER_HALF_WIDTH;
                    player.position.y = player.aabb.mins.y;
                    player.position.z = player.aabb.mins.z + PLAYER_HALF_WIDTH;

                    player.acceleration.x = 0.0;
                    player.acceleration.y = 0.0;
                    player.acceleration.z = 0.0;

                    player
                },
            )
        }
    }
}
