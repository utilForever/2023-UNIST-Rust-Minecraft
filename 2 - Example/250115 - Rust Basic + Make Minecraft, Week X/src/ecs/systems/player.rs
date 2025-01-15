use crate::aabb::{get_block_aabb, AABB};
use crate::chunk::BlockID;
use crate::chunk_manager::ChunkManager;
use crate::constants::{
    FAR_PLANE, FLYING_TRIGGER_INTERVAL, FOV, JUMP_IMPULSE, NEAR_PLANE, PLAYER_EYES_HEIGHT,
    REACH_DISTANCE, SPRINTING_TRIGGER_INTERVAL, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::input::InputCache;
use crate::inventory::Inventory;
use crate::particle_system::ParticleSystem;
use crate::physics::Interpolator;
use crate::player::{PlayerPhysicsState, PlayerState};
use crate::raycast;
use crate::timer::Timer;
use crate::types::{ParticleSystems, TexturePack};
use crate::util::Forward;
use nalgebra::Vector3;
use nalgebra_glm::{vec3, IVec3};
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};
use std::sync::Arc;
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
            let player_physics_state = player_physics_state.get_latest_state_mut();

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
                        if player_state.is_on_ground {
                            player_physics_state.velocity.y = *JUMP_IMPULSE;
                            player_state.jump_last_executed = Instant::now();
                        }
                    }

                    // Cancel sneaking
                    glfw::WindowEvent::Key(glfw::Key::LeftShift, _, glfw::Action::Release, _) => {
                        player_state.is_sneaking = false;
                    }

                    // Cancel sprinting
                    glfw::WindowEvent::Key(glfw::Key::W, _, glfw::Action::Release, _) => {
                        player_state.is_sprinting = false;
                    }

                    // Sprint on double press
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

            // Sneaking
            if input_cache.is_key_pressed(glfw::Key::LeftShift) && player_state.is_on_ground {
                player_state.is_sneaking = true;
                player_state.is_sprinting = false;
            }

            // Sprinting
            if input_cache.is_key_pressed(glfw::Key::LeftControl)
                && input_cache.is_key_pressed(glfw::Key::W)
                && !player_state.is_sneaking
            {
                player_state.is_sprinting = true;
            }
        }
    }
}

pub struct UpdatePlayerState;

impl<'a> System<'a> for UpdatePlayerState {
    type SystemData = (
        Read<'a, Timer>,
        Write<'a, Arc<ChunkManager>>,
        WriteStorage<'a, PlayerState>,
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (global_timer, chunk_manager, mut player_state, player_physics_state) = data;

        for (player_state, player_physics_state) in
            (&mut player_state, &player_physics_state).join()
        {
            let t = global_timer.time();

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

            // Targeted block
            player_state.targeted_block = {
                let is_solid_block_at =
                    |x: i32, y: i32, z: i32| chunk_manager.is_solid_block_at(x, y, z);

                let forward = player_state.rotation.forward();
                let player = player_physics_state.get_interpolated_state();

                raycast::raycast(
                    &is_solid_block_at,
                    &(player.position
                        + vec3(
                            0.0,
                            *player_state.camera_height.get_interpolated_state(),
                            0.0,
                        )),
                    &forward.normalize(),
                    REACH_DISTANCE,
                )
            };

            // View and projection matrix
            player_state.view_matrix = {
                let player_physics_state = player_physics_state.get_interpolated_state();
                let camera_position = player_physics_state.position
                    + vec3(
                        0.0,
                        *player_state.camera_height.get_interpolated_state(),
                        0.0,
                    );
                let looking_dir = player_state.rotation.forward();

                nalgebra_glm::look_at(
                    &camera_position,
                    &(camera_position + looking_dir),
                    &Vector3::y(),
                )
            };

            player_state.projection_matrix = {
                let fov = *player_state.fov.get_interpolated_state();
                nalgebra_glm::perspective(
                    WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                    fov,
                    NEAR_PLANE,
                    FAR_PLANE,
                )
            };
        }
    }
}

pub struct PlaceAndBreakBlocks;

impl<'a> System<'a> for PlaceAndBreakBlocks {
    type SystemData = (
        Write<'a, Arc<ChunkManager>>,
        Write<'a, ParticleSystems>,
        Read<'a, InputCache>,
        Read<'a, TexturePack>,
        WriteStorage<'a, PlayerState>,
        ReadStorage<'a, Interpolator<PlayerPhysicsState>>,
        ReadStorage<'a, Inventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            chunk_manager,
            mut particle_systems,
            input_cache,
            texture_pack,
            mut player_state,
            player_physics_state,
            inventory,
        ) = data;

        for (player_state, player_physics_state, inventory) in
            (&mut player_state, &player_physics_state, &inventory).join()
        {
            let player_physics_state = player_physics_state.get_latest_state();

            for event in &input_cache.events {
                match event {
                    glfw::WindowEvent::MouseButton(button, glfw::Action::Press, _) => {
                        player_state.block_placing_last_executed = Instant::now();

                        match button {
                            glfw::MouseButton::Button1 => {
                                if let Some(((x, y, z), _)) = &player_state.targeted_block {
                                    let mut particle_system =
                                        particle_systems.get_mut("block_particles").unwrap();
                                    break_block(
                                        (*x, *y, *z),
                                        &chunk_manager,
                                        &mut particle_system,
                                        &texture_pack,
                                    );
                                }
                            }
                            glfw::MouseButton::Button2 => {
                                if let Some(((x, y, z), normal)) = &player_state.targeted_block {
                                    place_block(
                                        (*x, *y, *z),
                                        &normal,
                                        &player_physics_state.aabb,
                                        &inventory,
                                        &chunk_manager,
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Repeated block placing or breaking while the mouse button is pressed
            {
                let now = Instant::now();

                if now
                    .duration_since(player_state.block_placing_last_executed)
                    .as_secs_f32()
                    >= 0.25
                {
                    if input_cache.is_mouse_button_pressed(glfw::MouseButtonLeft) {
                        if let Some(((x, y, z), _)) = &player_state.targeted_block {
                            let mut particle_system =
                                particle_systems.get_mut("block_particles").unwrap();
                            break_block(
                                (*x, *y, *z),
                                &chunk_manager,
                                &mut particle_system,
                                &texture_pack,
                            );
                        }

                        player_state.block_placing_last_executed = now;
                    } else if input_cache.is_mouse_button_pressed(glfw::MouseButtonRight) {
                        if let Some(((x, y, z), normal)) = &player_state.targeted_block {
                            place_block(
                                (*x, *y, *z),
                                &normal,
                                &player_physics_state.aabb,
                                &inventory,
                                &chunk_manager,
                            );
                        }

                        player_state.block_placing_last_executed = now;
                    }
                }
            }
        }
    }
}

fn break_block(
    (x, y, z): (i32, i32, i32),
    chunk_manager: &ChunkManager,
    particle_system: &mut ParticleSystem,
    uv_map: &TexturePack,
) {
    let block = chunk_manager.get_block(x, y, z).unwrap();

    if block != BlockID::Air {
        chunk_manager.put_block(x, y, z, BlockID::Air);
        particle_system.spawn_block_breaking_particles(
            vec3(x as f32, y as f32, z as f32),
            &uv_map,
            block,
        );

        info!("Destroyed block at ({x} {y} {z})");
    }
}

fn place_block(
    (x, y, z): (i32, i32, i32),
    normal: &IVec3,
    player_aabb: &AABB,
    inventory: &Inventory,
    chunk_manager: &ChunkManager,
) {
    let adjacent_block = IVec3::new(x, y, z) + normal;
    let adjacent_block_aabb = get_block_aabb(&vec3(
        adjacent_block.x as f32,
        adjacent_block.y as f32,
        adjacent_block.z as f32,
    ));

    if !player_aabb.intersects(&adjacent_block_aabb) {
        if let Some(block) = inventory.get_selected_item() {
            chunk_manager.put_block(adjacent_block.x, adjacent_block.y, adjacent_block.z, block);
        }

        info!(
            "Put block at {} {} {}",
            adjacent_block.x, adjacent_block.y, adjacent_block.z
        );
    }
}
