#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod debugging;
pub mod aabb;
pub mod block_texture_sides;
pub mod chunk;
pub mod chunk_manager;
pub mod constants;
pub mod ecs;
pub mod physics;
pub mod raycast;
pub mod renderer;
pub mod shader;
pub mod shapes;
pub mod texture;
pub mod util;

use crate::aabb::AABB;
use crate::block_texture_sides::BlockFaces;
use crate::chunk::BlockID;
use crate::chunk_manager::ChunkManager;
use crate::debugging::*;
use crate::physics::{PhysicsManager, PlayerPhysicsState};
use crate::shader::{ShaderPart, ShaderProgram};
use crate::util::forward;
// use glfw::ffi::glfwSwapInterval;
use crate::constants::*;
use glfw::{Action, Context, CursorMode, Key, MouseButton, OpenGlProfileHint, WindowHint};
use image::{ColorType, DynamicImage};
use nalgebra::{clamp, Vector3};
use nalgebra_glm::{pi, vec2, vec3, IVec3, Vec2, Vec3};
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_void;

type UVCoords = (f32, f32, f32, f32);
type UVFaces = (UVCoords, UVCoords, UVCoords, UVCoords, UVCoords, UVCoords);

pub struct InputCache {
    pub last_cursor_pos: Vec2,
    pub cursor_rel_pos: Vec2,
    pub key_states: HashMap<Key, Action>,
}

impl Default for InputCache {
    fn default() -> Self {
        InputCache {
            last_cursor_pos: vec2(0.0, 0.0),
            cursor_rel_pos: vec2(0.0, 0.0),
            key_states: HashMap::new(),
        }
    }
}

impl InputCache {
    pub fn is_key_pressed(&self, key: Key) -> bool {
        match self.key_states.get(&key) {
            Some(action) => *action == Action::Press || *action == Action::Repeat,
            None => false,
        }
    }
}

pub struct PlayerRenderState {
    pub rotation: Vec3,
}

impl PlayerRenderState {
    pub fn new() -> Self {
        Self {
            rotation: vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn get_camera_rotation(&mut self) -> &mut Vec3 {
        &mut self.rotation
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersionMajor(OPENGL_MAJOR_VERSION));
    glfw.window_hint(WindowHint::ContextVersionMinor(OPENGL_MINOR_VERSION));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WINDOW_NAME,
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_raw_mouse_motion(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    // unsafe { glfwSwapInterval(0) };

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
    gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
    gl_call!(gl::Viewport(
        0,
        0,
        WINDOW_WIDTH as i32,
        WINDOW_HEIGHT as i32
    ));

    let mut player_render_state = PlayerRenderState::new();
    let mut physics_manager = PhysicsManager::new(
        1.0 / PHYSICS_TICKRATE,
        PlayerPhysicsState::new_at_position(vec3(0.0f32, 30.0, 0.0)),
    );

    let vert =
        ShaderPart::from_vert_source(&CString::new(include_str!("shaders/diffuse.vert")).unwrap())
            .unwrap();
    let frag =
        ShaderPart::from_frag_source(&CString::new(include_str!("shaders/diffuse.frag")).unwrap())
            .unwrap();
    let mut program = ShaderProgram::from_shaders(vert, frag).unwrap();

    // Generate texture atlas
    let mut texture_map: HashMap<BlockID, BlockFaces<&str>> = HashMap::new();
    texture_map.insert(BlockID::Dirt, BlockFaces::All("blocks/dirt.png"));
    texture_map.insert(
        BlockID::GrassBlock,
        BlockFaces::Sides {
            sides: "blocks/grass_block_side.png",
            top: "blocks/grass_block_top.png",
            bottom: "blocks/dirt.png",
        },
    );
    texture_map.insert(
        BlockID::Cobblestone,
        BlockFaces::All("blocks/cobblestone.png"),
    );
    texture_map.insert(BlockID::Obsidian, BlockFaces::All("blocks/obsidian.png"));
    texture_map.insert(
        BlockID::OakLog,
        BlockFaces::Sides {
            sides: "blocks/oak_log.png",
            top: "blocks/oak_log_top.png",
            bottom: "blocks/oak_log_top.png",
        },
    );
    texture_map.insert(BlockID::OakLeaves, BlockFaces::All("blocks/oak_leaves.png"));
    texture_map.insert(BlockID::Debug, BlockFaces::All("blocks/debug.png"));
    texture_map.insert(BlockID::Debug2, BlockFaces::All("blocks/debug2.png"));

    let mut atlas = 0;
    gl_call!(gl::CreateTextures(gl::TEXTURE_2D, 1, &mut atlas));
    gl_call!(gl::TextureParameteri(
        atlas,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST_MIPMAP_NEAREST as i32
    ));
    gl_call!(gl::TextureParameteri(
        atlas,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));
    gl_call!(gl::TextureStorage2D(
        atlas,
        1,
        gl::RGBA8,
        TEXTURE_ATLAS_SIZE as i32,
        TEXTURE_ATLAS_SIZE as i32
    ));

    let mut uv_map = HashMap::<BlockID, BlockFaces<UVCoords>>::new();
    let mut x = 0;
    let mut y = 0;

    let load_image = |texture_path: &str| {
        let img = image::open(texture_path);
        let img = match img {
            Ok(img) => img.flipv(),
            Err(err) => panic!("Filename: {texture_path}, error: {err}"),
        };

        match img.color() {
            ColorType::Rgba8 => {}
            _ => panic!("Texture format not supported"),
        };

        img
    };

    let mut blit_image = |img: &mut DynamicImage| {
        let (dest_x, dest_y) = (x, y);

        gl_call!(gl::TextureSubImage2D(
            atlas,
            0,
            dest_x as i32,
            dest_y as i32,
            img.width() as i32,
            img.height() as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.as_bytes().as_ptr() as *mut c_void
        ));

        x += BLOCK_TEXTURE_SIZE;

        if x >= TEXTURE_ATLAS_SIZE {
            x = 0;
            y += BLOCK_TEXTURE_SIZE;
        }

        let (dest_x, dest_y) = (dest_x as f32, dest_y as f32);
        (
            dest_x / TEXTURE_ATLAS_SIZE as f32,
            dest_y / TEXTURE_ATLAS_SIZE as f32,
            (dest_x + BLOCK_TEXTURE_SIZE as f32) / TEXTURE_ATLAS_SIZE as f32,
            (dest_y + BLOCK_TEXTURE_SIZE as f32) / TEXTURE_ATLAS_SIZE as f32,
        )
    };

    for (block, faces) in texture_map {
        match faces {
            BlockFaces::All(all) => {
                let mut img = load_image(all);
                let uv = blit_image(&mut img);
                uv_map.insert(block, BlockFaces::All(uv));
            }
            BlockFaces::Sides { sides, top, bottom } => {
                let mut img = load_image(sides);
                let uv_sides = blit_image(&mut img);

                let mut img = load_image(top);
                let uv_top = blit_image(&mut img);

                let mut img = load_image(bottom);
                let uv_bottom = blit_image(&mut img);

                uv_map.insert(
                    block,
                    BlockFaces::Sides {
                        sides: uv_sides,
                        top: uv_top,
                        bottom: uv_bottom,
                    },
                );
            }
            BlockFaces::Each {
                top,
                bottom,
                front,
                back,
                left,
                right,
            } => {
                let mut img = load_image(top);
                let uv_top = blit_image(&mut img);

                let mut img = load_image(bottom);
                let uv_bottom = blit_image(&mut img);

                let mut img = load_image(front);
                let uv_front = blit_image(&mut img);

                let mut img = load_image(back);
                let uv_back = blit_image(&mut img);

                let mut img = load_image(left);
                let uv_left = blit_image(&mut img);

                let mut img = load_image(right);
                let uv_right = blit_image(&mut img);

                uv_map.insert(
                    block,
                    BlockFaces::Each {
                        top: uv_top,
                        bottom: uv_bottom,
                        front: uv_front,
                        back: uv_back,
                        left: uv_left,
                        right: uv_right,
                    },
                );
            }
        }
    }

    gl_call!(gl::ActiveTexture(gl::TEXTURE0));
    gl_call!(gl::BindTexture(gl::TEXTURE_2D, atlas));

    let mut chunk_manager = ChunkManager::default();
    chunk_manager.simplex();

    let mut input_cache = InputCache::default();
    let mut prev_cursor_pos = (0.0, 0.0);

    // Loop until the user closes the window
    while !window.should_close() {
        // Poll and process events
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::CursorPos(x, y) => {
                    let rel_x = x - prev_cursor_pos.0;
                    let rel_y = y - prev_cursor_pos.1;

                    player_render_state.rotation.y += rel_x as f32 / 100.0 * MOUSE_SENSITIVITY_X;
                    player_render_state.rotation.x += rel_y as f32 / 100.0 * MOUSE_SENSITIVITY_Y;

                    player_render_state.rotation.x = clamp(
                        player_render_state.rotation.x,
                        -pi::<f32>() / 2.0 + 0.0001,
                        pi::<f32>() / 2.0 - 0.0001,
                    );

                    prev_cursor_pos = (x, y);
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(key, _, action, _) => {
                    input_cache.key_states.insert(key, action);
                }
                glfw::WindowEvent::MouseButton(button, Action::Press, _) => {
                    let forward = forward(&player_render_state.rotation);
                    let get_voxel = |x: i32, y: i32, z: i32| {
                        chunk_manager
                            .get_block(x, y, z)
                            .filter(|&block| block != BlockID::Air)
                            .map(|_| (x, y, z))
                    };

                    let player = physics_manager.get_current_state();
                    let hit = raycast::raycast(
                        &get_voxel,
                        &player.get_camera_position(),
                        &forward.normalize(),
                        REACH_DISTANCE,
                    );

                    if let Some(((x, y, z), normal)) = hit {
                        if button == MouseButton::Button1 {
                            chunk_manager.set_block(x, y, z, BlockID::Air)
                        } else if button == MouseButton::Button2 {
                            let near = IVec3::new(x, y, z) + normal;

                            if !player.aabb.intersects(&get_block_aabb(&vec3(
                                near.x as f32,
                                near.y as f32,
                                near.z as f32,
                            ))) {
                                chunk_manager.set_block(near.x, near.y, near.z, BlockID::Debug2);
                                println!("Put block at {} {} {}", near.x, near.y, near.z);
                            }
                        }

                        println!("HIT {} {} {}", x, y, z);
                        dbg!(forward);
                    } else {
                        println!("No hit");
                    }
                }
                _ => {}
            }
        }

        let mut rotation = player_render_state.rotation;
        rotation.x = 0.0;

        use crate::physics::get_block_aabb;
        use num_traits::identities::Zero;

        let render_state = physics_manager.step(
            &|mut previous_state: PlayerPhysicsState, _t: f32, dt: f32| {
                let player = &mut previous_state;

                if input_cache.is_key_pressed(Key::Space) {
                    if player.is_on_ground {
                        player.velocity.y = *JUMP_IMPULSE;
                    }
                }

                let mut directional_acceleration = vec3(0.0, 0.0, 0.0);

                if input_cache.is_key_pressed(Key::W) {
                    directional_acceleration += forward(&rotation);
                }

                if input_cache.is_key_pressed(Key::S) {
                    directional_acceleration -= forward(&rotation);
                }

                if input_cache.is_key_pressed(Key::A) {
                    directional_acceleration -= forward(&rotation).cross(&Vector3::y());
                }

                if input_cache.is_key_pressed(Key::D) {
                    directional_acceleration += forward(&rotation).cross(&Vector3::y());
                }

                if directional_acceleration.norm_squared() != 0.0 {
                    let directional_acceleration = directional_acceleration
                        .normalize()
                        .scale(HORIZONTAL_ACCELERATION);
                    player.acceleration = directional_acceleration;
                }

                player.acceleration.y = GRAVITY;
                player.velocity += player.acceleration * dt;

                // Horizontal
                let mut horizontal = vec2(player.velocity.x, player.velocity.z);
                let mag = horizontal.magnitude();

                if mag > WALKING_SPEED {
                    horizontal = horizontal.scale(WALKING_SPEED / mag);
                }

                // Vertical
                // NOTE: https://www.planetminecraft.com/blog/the-acceleration-of-gravity-in-minecraft-and-terminal-velocity/
                if player.velocity.y < -MAX_VERTICAL_VELOCITY {
                    player.velocity.y = -MAX_VERTICAL_VELOCITY;
                }

                player.velocity.x = horizontal.x;
                player.velocity.z = horizontal.y;

                let mut is_player_on_ground = false;

                let separated_axis = &[
                    vec3(player.velocity.x, 0.0, 0.0),
                    vec3(0.0, player.velocity.y, 0.0),
                    vec3(0.0, 0.0, player.velocity.z),
                ];

                for v in separated_axis {
                    player.aabb.translate(&(v * dt));

                    let player_mins = &player.aabb.mins;
                    let player_maxs = &player.aabb.maxs;

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

                                    let block_aabb =
                                        get_block_aabb(&vec3(x as f32, y as f32, z as f32));

                                    if player.aabb.intersects(&block_aabb) {
                                        block_collided = Some(vec3(x as f32, y as f32, z as f32));
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }

                    // Reaction
                    if let Some(block_collided) = block_collided {
                        let block_aabb = get_block_aabb(&block_collided);

                        if !v.x.is_zero() {
                            if v.x < 0.0 {
                                player.aabb = AABB::new(
                                    vec3(block_aabb.maxs.x, player.aabb.mins.y, player.aabb.mins.z),
                                    vec3(
                                        block_aabb.maxs.x + PLAYER_WIDTH,
                                        player.aabb.maxs.y,
                                        player.aabb.maxs.z,
                                    ),
                                )
                            } else {
                                player.aabb = AABB::new(
                                    vec3(
                                        block_aabb.mins.x - PLAYER_WIDTH,
                                        player.aabb.mins.y,
                                        player.aabb.mins.z,
                                    ),
                                    vec3(block_aabb.mins.x, player.aabb.maxs.y, player.aabb.maxs.z),
                                )
                            }

                            player.velocity.x = 0.0;
                        }

                        if !v.y.is_zero() {
                            if v.y < 0.0 {
                                player.aabb = AABB::new(
                                    vec3(player.aabb.mins.x, block_aabb.maxs.y, player.aabb.mins.z),
                                    vec3(
                                        player.aabb.maxs.x,
                                        block_aabb.maxs.y + PLAYER_HEIGHT,
                                        player.aabb.maxs.z,
                                    ),
                                );
                                is_player_on_ground = true;
                            } else {
                                player.aabb = AABB::new(
                                    vec3(
                                        player.aabb.mins.x,
                                        block_aabb.mins.y - PLAYER_HEIGHT,
                                        player.aabb.mins.z,
                                    ),
                                    vec3(player.aabb.maxs.x, block_aabb.mins.y, player.aabb.maxs.z),
                                )
                            }

                            player.velocity.y = 0.0;
                        }

                        if !v.z.is_zero() {
                            if v.z < 0.0 {
                                player.aabb = AABB::new(
                                    vec3(player.aabb.mins.x, player.aabb.mins.y, block_aabb.maxs.z),
                                    vec3(
                                        player.aabb.maxs.x,
                                        player.aabb.maxs.y,
                                        block_aabb.maxs.z + PLAYER_WIDTH,
                                    ),
                                )
                            } else {
                                player.aabb = AABB::new(
                                    vec3(
                                        player.aabb.mins.x,
                                        player.aabb.mins.y,
                                        block_aabb.mins.z - PLAYER_WIDTH,
                                    ),
                                    vec3(player.aabb.maxs.x, player.aabb.maxs.y, block_aabb.mins.z),
                                )
                            }

                            player.velocity.z = 0.0;
                        }
                    }
                }

                player.position.x = player.aabb.mins.x + PLAYER_HALF_WIDTH;
                player.position.y = player.aabb.mins.y;
                player.position.z = player.aabb.mins.z + PLAYER_HALF_WIDTH;

                player.is_on_ground = is_player_on_ground;

                let friction = if is_player_on_ground {
                    ON_GROUND_FRICTION
                } else {
                    IN_AIR_FRICTION
                };

                if player.acceleration.x.is_zero()
                    || player.acceleration.x.signum() != player.velocity.x.signum()
                {
                    player.velocity.x -= friction * player.velocity.x * dt;
                }

                if player.acceleration.z.is_zero()
                    || player.acceleration.z.signum() != player.velocity.z.signum()
                {
                    player.velocity.z -= friction * player.velocity.z * dt;
                }

                player.acceleration.x = 0.0;
                player.acceleration.y = 0.0;
                player.acceleration.z = 0.0;

                previous_state
            },
        );
        let player = &render_state;

        let direction = forward(&player_render_state.rotation);
        let camera_position = player.get_camera_position();
        let view_matrix = nalgebra_glm::look_at(
            &camera_position,
            &(camera_position + direction),
            &Vector3::y(),
        );
        let projection_matrix =
            nalgebra_glm::perspective(1.0, pi::<f32>() / 2.0, NEAR_PLANE, FAR_PLANE);

        chunk_manager.rebuild_dirty_chunks(&uv_map);

        program.use_program();
        unsafe {
            program.set_uniform_matrix4fv("view", view_matrix.as_ptr());
            program.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
        }
        program.set_uniform1i("tex", 0);

        let (r, g, b, a) = BACKGROUND_COLOR;
        gl_call!(gl::ClearColor(r, g, b, a));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        chunk_manager.render_loaded_chunks(&mut program);

        // Swap front and back buffers
        window.swap_buffers();
    }
}
