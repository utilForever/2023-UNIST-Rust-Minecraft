use crate::input::InputCache;
use crate::timer::Timer;
use glfw::{Action, Context, Glfw, Key, Window, WindowEvent};
use specs::{System, Write};
use std::process::exit;
use std::sync::mpsc::Receiver;

pub struct ReadWindowEvents {
    pub glfw: Glfw,
    pub window: Window,
    pub events: Receiver<(f64, WindowEvent)>,
}

impl<'a> System<'a> for ReadWindowEvents {
    type SystemData = (Write<'a, InputCache>, Write<'a, Timer>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut input_cache, mut global_timer) = data;

        if self.window.should_close() {
            exit(0);
        }

        self.window.swap_buffers();

        input_cache.events.clear();
        self.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            input_cache.handle_event(&event);

            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }
                WindowEvent::Key(Key::P, _, Action::Press, _) => {
                    if global_timer.is_paused() {
                        global_timer.resume();
                    } else {
                        global_timer.pause();
                    }
                }
                _ => {}
            }
        }
    }
}
