use glfw::{Action, Key, MouseButton, WindowEvent};
use nalgebra_glm::{vec2, DVec2};
use std::collections::HashMap;

pub struct InputCache {
    pub events: Vec<WindowEvent>,
    pub last_cursor_pos: DVec2,
    pub cursor_rel_pos: DVec2,
    pub key_states: HashMap<Key, Action>,
    pub mouse_button_states: HashMap<MouseButton, Action>,
}

impl Default for InputCache {
    fn default() -> Self {
        InputCache {
            events: Vec::new(),
            last_cursor_pos: vec2(0.0, 0.0),
            cursor_rel_pos: vec2(0.0, 0.0),
            key_states: HashMap::default(),
            mouse_button_states: HashMap::default(),
        }
    }
}

impl InputCache {
    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.events.push(event.clone());

        match event {
            &glfw::WindowEvent::CursorPos(x, y) => {
                self.cursor_rel_pos.x = x - self.last_cursor_pos.x;
                self.cursor_rel_pos.y = y - self.last_cursor_pos.y;
                self.last_cursor_pos.x = x;
                self.last_cursor_pos.y = y;
            }
            &glfw::WindowEvent::Key(key, _, action, _) => {
                self.key_states.insert(key, action);
            }
            &glfw::WindowEvent::MouseButton(button, action, _) => {
                self.mouse_button_states.insert(button, action);
            }
            _ => {}
        }
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        match self.key_states.get(&key) {
            Some(action) => *action == Action::Press || *action == Action::Repeat,
            None => false,
        }
    }

    pub fn is_mouse_button_pressed(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_states
            .get(&mouse_button)
            .filter(|&&a| a == Action::Press || a == Action::Repeat)
            .is_some()
    }
}
