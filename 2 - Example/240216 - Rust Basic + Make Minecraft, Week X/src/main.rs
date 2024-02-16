#[macro_use]
pub mod debugging;
pub mod chunk;
pub mod chunk_manager;
pub mod ecs;
pub mod raycast;
pub mod renderer;
pub mod shader;
pub mod shapes;
pub mod texture;
pub mod util;

use crate::chunk::BlockID;
use crate::chunk_manager::ChunkManager;
use crate::debugging::*;
use crate::shader::{ShaderPart, ShaderProgram};
use crate::util::forward;
use glfw::ffi::glfwSwapInterval;
use glfw::{Action, Context, CursorMode, Key, MouseButton, OpenGlProfileHint, WindowHint};
use image::ColorType;
use nalgebra::{clamp, Vector3};
use nalgebra_glm::{pi, vec2, vec3, IVec3, Vec2};
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_void;

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

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(6));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));

    let window_size = (800, 800);
    let window_title = "Minecraft";

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(
            window_size.0,
            window_size.1,
            window_title,
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_raw_mouse_motion(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_mode(CursorMode::Normal);
    window.set_cursor_pos(400.0, 400.0);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    unsafe { glfwSwapInterval(0) };

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
    gl_call!(gl::Viewport(0, 0, 800, 800));

    let mut camera_position = vec3(0.0f32, 0.0, 0.0);
    let mut camera_rotation = vec3(0.0f32, 0.0, 0.0);

    let vert =
        ShaderPart::from_vert_source(&CString::new(include_str!("shaders/diffuse.vert")).unwrap())
            .unwrap();
    let frag =
        ShaderPart::from_frag_source(&CString::new(include_str!("shaders/diffuse.frag")).unwrap())
            .unwrap();
    let mut program = ShaderProgram::from_shaders(vert, frag).unwrap();

    // Generate texture atlas
    let mut texture_map: HashMap<BlockID, &str> = HashMap::new();
    texture_map.insert(BlockID::DIRT, "blocks/dirt.png");
    texture_map.insert(BlockID::COBBLESTONE, "blocks/cobblestone.png");
    texture_map.insert(BlockID::OBSIDIAN, "blocks/obsidian.png");

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
    gl_call!(gl::TextureStorage2D(atlas, 1, gl::RGBA8, 1024, 1024,));

    let mut uv_map = HashMap::<BlockID, ((f32, f32), (f32, f32))>::new();
    let mut x = 0;
    let mut y = 0;

    for (block, texture_path) in texture_map {
        let img = image::open(texture_path);
        let img = match img {
            Ok(img) => img.flipv(),
            Err(err) => panic!("Filename: {texture_path}, error: {err}"),
        };

        match img.color() {
            ColorType::Rgba8 => {}
            _ => panic!("Texture format not supported"),
        };

        gl_call!(gl::TextureSubImage2D(
            atlas,
            0,
            x,
            y,
            img.width() as i32,
            img.height() as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.as_bytes().as_ptr() as *mut c_void
        ));

        uv_map.insert(
            block,
            (
                (x as f32 / 1024.0, y as f32 / 1024.0),
                ((x as f32 + 16.0) / 1024.0, (y as f32 + 16.0) / 1024.0),
            ),
        );

        x += 16;

        if x >= 1024 {
            x = 0;
            y += 16;
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

                    camera_rotation.y += rel_x as f32 / 100.0;
                    camera_rotation.x += rel_y as f32 / 100.0;

                    camera_rotation.x = clamp(
                        camera_rotation.x,
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
                    let forward = forward(&camera_rotation);
                    let get_voxel = |x: i32, y: i32, z: i32| {
                        chunk_manager
                            .get_block(x, y, z)
                            .filter(|&block| block != BlockID::AIR)
                            .map(|_| (x, y, z))
                    };

                    let hit =
                        raycast::raycast(&get_voxel, &camera_position, &forward.normalize(), 400.0);

                    if let Some(((x, y, z), normal)) = hit {
                        if button == MouseButton::Button1 {
                            chunk_manager.set_block(x, y, z, BlockID::AIR)
                        } else if button == MouseButton::Button2 {
                            let near = IVec3::new(x, y, z) + normal;
                            chunk_manager.set_block(near.x, near.y, near.z, BlockID::DIRT)
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

        let multiplier = 0.01f32;

        if input_cache.is_key_pressed(Key::W) {
            camera_position += forward(&camera_rotation).scale(multiplier);
        }

        if input_cache.is_key_pressed(Key::S) {
            camera_position -= forward(&camera_rotation).scale(multiplier);
        }

        if input_cache.is_key_pressed(Key::A) {
            camera_position -= forward(&camera_rotation)
                .cross(&Vector3::y())
                .scale(multiplier);
        }

        if input_cache.is_key_pressed(Key::D) {
            camera_position += forward(&camera_rotation)
                .cross(&Vector3::y())
                .scale(multiplier);
        }

        if input_cache.is_key_pressed(Key::Q) {
            camera_position.y += multiplier;
        }

        if input_cache.is_key_pressed(Key::Z) {
            camera_position.y -= multiplier;
        }

        let direction = forward(&camera_rotation);

        let view_matrix = nalgebra_glm::look_at(
            &camera_position,
            &(camera_position + direction),
            &Vector3::y(),
        );
        let projection_matrix = nalgebra_glm::perspective(1.0, pi::<f32>() / 2.0, 0.1, 1000.0);

        chunk_manager.rebuild_dirty_chunks(&uv_map);

        program.use_program();
        unsafe {
            program.set_uniform_matrix4fv("view", view_matrix.as_ptr());
            program.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
        }
        program.set_uniform1i("tex", 0);

        gl_call!(gl::ClearColor(0.74, 0.84, 1.0, 1.0));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        chunk_manager.render_loaded_chunks(&mut program);

        // Swap front and back buffers
        window.swap_buffers();
    }
}
