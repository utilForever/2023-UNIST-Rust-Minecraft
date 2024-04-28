use crate::constants::{OPENGL_MAJOR_VERSION, OPENGL_MINOR_VERSION};
use glfw::{Context, CursorMode, Glfw, OpenGlProfileHint, Window, WindowEvent, WindowHint};
use std::sync::mpsc::Receiver;

pub fn create_window(
    width: u32,
    height: u32,
    title: &str,
) -> (Glfw, Window, Receiver<(f64, WindowEvent)>) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersionMajor(OPENGL_MAJOR_VERSION));
    glfw.window_hint(WindowHint::ContextVersionMinor(OPENGL_MINOR_VERSION));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));
    // Uncomment the following line to disable VSync
    // unsafe { glfwSwapInterval(0) };

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(width, height, title, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_raw_mouse_motion(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);

    (glfw, window, events)
}
