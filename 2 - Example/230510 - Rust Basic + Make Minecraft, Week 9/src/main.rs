use glfw::ffi::glfwSwapInterval;
use glfw::{Context, OpenGlProfileHint, WindowHint};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(6));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));

    let window_size = (500, 500);
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

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    unsafe { glfwSwapInterval(0) };

    // Loop until the user closes the window
    while !window.should_close() {
        // Poll and process events
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            println!("{event:?}");
        }

        // Swap front and back buffers
        window.swap_buffers();
    }
}
