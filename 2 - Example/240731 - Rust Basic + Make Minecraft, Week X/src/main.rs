#[macro_use]
extern crate lazy_static;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[macro_use]
pub mod debugging;
pub mod aabb;
pub mod ambient_occlusion;
pub mod block_texture_faces;
pub mod chunk;
pub mod chunk_manager;
pub mod constants;
pub mod drawing;
pub mod ecs;
pub mod fps_counter;
pub mod gui;
pub mod input;
pub mod inventory;
pub mod particle_system;
pub mod physics;
pub mod player;
pub mod raycast;
pub mod renderer;
pub mod shader;
pub mod shapes;
pub mod texture;
pub mod texture_pack;
pub mod timer;
pub mod types;
pub mod util;
pub mod window;

use crate::aabb::{get_block_aabb, AABB};
use crate::chunk::BlockID;
use crate::chunk_manager::ChunkManager;
use crate::debugging::*;
use crate::physics::Interpolator;
use crate::shader::ShaderProgram;
use std::collections::HashMap;
// use glfw::ffi::glfwSwapInterval;
use crate::constants::*;
use crate::fps_counter::FpsCounter;
use crate::gui::{
    create_block_outline_vao, create_crosshair_vao, create_gui_icons_texture,
    create_hotbar_selection_vao, create_hotbar_vao, create_widgets_texture, draw_crosshair,
};
use crate::input::InputCache;
use crate::inventory::Inventory;
use crate::particle_system::ParticleSystem;
use crate::player::{PlayerPhysicsState, PlayerProperties};
use crate::texture_pack::generate_array_texture;
use crate::timer::Timer;
use crate::types::UVMap;
use crate::util::Forward;
use crate::window::create_window;
use glfw::{Action, Context, Key, MouseButton};
use nalgebra::{Matrix4, Vector3};
use nalgebra_glm::{vec3, IVec3};
use std::os::raw::c_void;
use std::time::Instant;

fn main() {
    let mut log_builder = pretty_env_logger::formatted_builder();
    log_builder.parse_filters(LOG_LEVEL.as_str()).init();

    let (mut glfw, mut window, events) = create_window(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_NAME);

    gl_call!(gl::Enable(gl::DEBUG_OUTPUT));
    gl_call!(gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS));
    gl_call!(gl::DebugMessageCallback(
        Some(debug_message_callback),
        std::ptr::null::<c_void>(),
    ));
    gl_call!(gl::DebugMessageControl(
        gl::DONT_CARE,
        gl::DONT_CARE,
        gl::DONT_CARE,
        0,
        std::ptr::null::<u32>(),
        gl::TRUE
    ));

    gl_call!(gl::Enable(gl::CULL_FACE));
    gl_call!(gl::CullFace(gl::BACK));
    gl_call!(gl::Enable(gl::DEPTH_TEST));
    gl_call!(gl::Enable(gl::BLEND));
    gl_call!(gl::Viewport(
        0,
        0,
        WINDOW_WIDTH as i32,
        WINDOW_HEIGHT as i32
    ));

    let (item_array_texture, uv_map) = generate_array_texture();
    gl_call!(gl::BindTextureUnit(0, item_array_texture));

    let gui_icons_texture = create_gui_icons_texture();
    gl_call!(gl::ActiveTexture(gl::TEXTURE1));
    gl_call!(gl::BindTexture(gl::TEXTURE_2D, gui_icons_texture));

    let gui_widgets_texture = create_widgets_texture();
    gl_call!(gl::ActiveTexture(gl::TEXTURE2));
    gl_call!(gl::BindTexture(gl::TEXTURE_2D, gui_widgets_texture));

    let mut voxel_shader =
        ShaderProgram::compile("src/shaders/voxel.vert", "src/shaders/voxel.frag");
    let mut gui_shader = ShaderProgram::compile("src/shaders/gui.vert", "src/shaders/gui.frag");
    let mut outline_shader =
        ShaderProgram::compile("src/shaders/outline.vert", "src/shaders/outline.frag");
    let mut item_shader = ShaderProgram::compile("src/shaders/item.vert", "src/shaders/item.frag");
    let mut particle_shader =
        ShaderProgram::compile("src/shaders/particle.vert", "src/shaders/particle.frag");

    let crosshair_vao = create_crosshair_vao();
    let block_outline_vao = create_block_outline_vao();
    let hotbar_vao = create_hotbar_vao();
    let hotbar_selection_vao = create_hotbar_selection_vao();

    let mut inventory = Inventory::new(&uv_map);

    let mut block_placing_last_executed = Instant::now();

    let mut player_properties = PlayerProperties::new();
    let mut player_physics_state = Interpolator::new(
        1.0 / PHYSICS_TICKRATE,
        PlayerPhysicsState::new_at_position(vec3(0.0f32, 30.0, 0.0)),
    );

    let mut global_timer = Timer::new();

    let mut chunk_manager = ChunkManager::default();
    chunk_manager.generate_terrain();
    chunk_manager.rebuild_dirty_chunks(&uv_map);

    let mut input_cache = InputCache::default();

    let mut particle_systems = HashMap::new();
    particle_systems.insert("block_particles", ParticleSystem::new(500));

    let mut fps_counter = FpsCounter::new();

    // Loop until the user closes the window
    while !window.should_close() {
        fps_counter.update();

        // Get looking block coords
        let looking_block = {
            let is_solid_block_at =
                |x: i32, y: i32, z: i32| chunk_manager.is_solid_block_at(x, y, z);

            let forward = player_properties.rotation.forward();
            let player = player_physics_state.get_interpolated_state();

            raycast::raycast(
                &is_solid_block_at,
                &(player.position
                    + vec3(
                        0.0,
                        *player_properties.camera_height.get_interpolated_state(),
                        0.0,
                    )),
                &forward.normalize(),
                REACH_DISTANCE,
            )
        };

        // Poll and process events
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            input_cache.handle_event(&event);
            inventory.handle_input_event(&event);

            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::P, _, Action::Press, _) => {
                    if global_timer.is_paused() {
                        global_timer.resume();
                    } else {
                        global_timer.pause();
                    }
                }
                glfw::WindowEvent::CursorPos(_, _) => {
                    player_properties.rotate_camera(
                        input_cache.cursor_rel_pos.x as f32,
                        input_cache.cursor_rel_pos.y as f32,
                    );
                }
                glfw::WindowEvent::MouseButton(button, Action::Press, _) => {
                    block_placing_last_executed = Instant::now();

                    match button {
                        MouseButton::Button1 => {
                            check_looking_block_and_break_block(
                                &looking_block,
                                &mut particle_systems,
                                &mut chunk_manager,
                                &uv_map,
                            );
                        }
                        MouseButton::Button2 => {
                            check_looking_block_and_place_block(
                                &looking_block,
                                &player_physics_state.get_latest_state(),
                                &inventory,
                                &mut chunk_manager,
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            player_properties.handle_input_event(&event);
            player_physics_state
                .get_latest_state()
                .handle_input_event(&event, &mut player_properties);
        }

        {
            let now = Instant::now();

            if now
                .duration_since(block_placing_last_executed)
                .as_secs_f32()
                >= 0.25
            {
                if input_cache.is_mouse_button_pressed(glfw::MouseButtonLeft) {
                    check_looking_block_and_break_block(
                        &looking_block,
                        &mut particle_systems,
                        &mut chunk_manager,
                        &uv_map,
                    );

                    block_placing_last_executed = Instant::now();
                } else if input_cache.is_mouse_button_pressed(glfw::MouseButtonRight) {
                    check_looking_block_and_place_block(
                        &looking_block,
                        &player_physics_state.get_latest_state(),
                        &inventory,
                        &mut chunk_manager,
                    );

                    block_placing_last_executed = Instant::now();
                }
            }
        }

        player_properties.on_update(
            global_timer.time(),
            &input_cache,
            &player_physics_state.get_latest_state(),
        );
        player_physics_state.update_player_physics(
            global_timer.time(),
            &input_cache,
            &chunk_manager,
            &mut player_properties,
        );

        let view_matrix = {
            let player_physics_state = player_physics_state.get_interpolated_state();
            let camera_position = player_physics_state.position
                + vec3(
                    0.0,
                    *player_properties.camera_height.get_interpolated_state(),
                    0.0,
                );
            let looking_dir = player_properties.rotation.forward();

            nalgebra_glm::look_at(
                &camera_position,
                &(camera_position + looking_dir),
                &Vector3::y(),
            )
        };

        let projection_matrix = {
            let fov = *player_properties.fov.get_interpolated_state();
            nalgebra_glm::perspective(
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                fov,
                NEAR_PLANE,
                FAR_PLANE,
            )
        };

        // Draw chunks
        {
            chunk_manager.rebuild_dirty_chunks(&uv_map);

            voxel_shader.use_program();
            unsafe {
                voxel_shader.set_uniform_matrix4fv("view", view_matrix.as_ptr());
                voxel_shader.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
            }
            voxel_shader.set_uniform1i("array_texture", 0);

            let (r, g, b, a) = BACKGROUND_COLOR;
            gl_call!(gl::ClearColor(r, g, b, a));
            gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
            gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

            chunk_manager.render_loaded_chunks(&mut voxel_shader);
        }

        // Particles
        {
            gl_call!(gl::Disable(gl::CULL_FACE));
            particle_shader.use_program();
            particle_shader.set_uniform1i("array_texture", 0);

            for particle_system in particle_systems.values_mut() {
                particle_system.update_all_particles(global_timer.time(), &chunk_manager);
                particle_system.render_all_particles(
                    &mut particle_shader,
                    &view_matrix,
                    &projection_matrix,
                );
            }
            gl_call!(gl::Enable(gl::CULL_FACE));
        }

        // Block outline
        {
            if let Some(((x, y, z), _)) = looking_block {
                let (x, y, z) = (x as f32, y as f32, z as f32);
                let model_matrix = Matrix4::new_translation(&vec3(x, y, z));

                outline_shader.use_program();
                unsafe {
                    outline_shader.set_uniform_matrix4fv("model", model_matrix.as_ptr());
                    outline_shader.set_uniform_matrix4fv("view", view_matrix.as_ptr());
                    outline_shader.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
                }

                gl_call!(gl::LineWidth(BLOCK_OUTLINE_WIDTH));
                gl_call!(gl::BindVertexArray(block_outline_vao));
                gl_call!(gl::DrawArrays(gl::LINES, 0, 24));
            }
        }

        // Draw GUI
        {
            draw_crosshair(crosshair_vao, &mut gui_shader);
            gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

            gl_call!(gl::Disable(gl::DEPTH_TEST));
            inventory.draw_hotbar(hotbar_vao, &mut gui_shader);
            inventory.draw_hotbar_selection_box(hotbar_selection_vao, &mut gui_shader);
            inventory.draw_hotbar_items(&mut item_shader);
            gl_call!(gl::Enable(gl::DEPTH_TEST));
        }

        // Swap front and back buffers
        window.swap_buffers();
    }
}

fn check_looking_block_and_break_block(
    looking_block: &Option<((i32, i32, i32), IVec3)>,
    particle_systems: &mut HashMap<&str, ParticleSystem>,
    chunk_manager: &mut ChunkManager,
    uv_map: &UVMap,
) {
    if let Some(((x, y, z), _)) = &looking_block {
        let mut particle_system = particle_systems.get_mut("block_particles").unwrap();
        break_block((*x, *y, *z), chunk_manager, &mut particle_system, &uv_map);
    }
}

fn break_block(
    (x, y, z): (i32, i32, i32),
    chunk_manager: &mut ChunkManager,
    particle_system: &mut ParticleSystem,
    uv_map: &UVMap,
) {
    let block = chunk_manager.get_block(x, y, z).unwrap();

    chunk_manager.set_block(x, y, z, BlockID::Air);
    particle_system.spawn_block_breaking_particles(
        vec3(x as f32, y as f32, z as f32),
        &uv_map,
        block,
    );

    info!("Destroyed block at ({x} {y} {z})");
}

fn check_looking_block_and_place_block(
    looking_block: &Option<((i32, i32, i32), IVec3)>,
    player_physics_state: &PlayerPhysicsState,
    inventory: &Inventory,
    chunk_manager: &mut ChunkManager,
) {
    if let Some(((x, y, z), normal)) = &looking_block {
        place_block(
            (*x, *y, *z),
            normal,
            &player_physics_state.aabb,
            &inventory,
            chunk_manager,
        );
    }
}

fn place_block(
    (x, y, z): (i32, i32, i32),
    normal: &IVec3,
    player_aabb: &AABB,
    inventory: &Inventory,
    chunk_manager: &mut ChunkManager,
) {
    let adjacent_block = IVec3::new(x, y, z) + normal;
    let adjacent_block_aabb = get_block_aabb(&vec3(
        adjacent_block.x as f32,
        adjacent_block.y as f32,
        adjacent_block.z as f32,
    ));

    if !player_aabb.intersects(&adjacent_block_aabb) {
        if let Some(block) = inventory.get_selected_item() {
            chunk_manager.set_block(adjacent_block.x, adjacent_block.y, adjacent_block.z, block);
        }

        info!(
            "Put block at {} {} {}",
            adjacent_block.x, adjacent_block.y, adjacent_block.z
        );
    }
}
