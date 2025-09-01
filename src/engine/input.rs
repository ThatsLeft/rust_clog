use sokol::app as sapp;
use glam::Vec2;

pub struct InputManager {
    keys_down: [bool; 512],
    keys_pressed: [bool; 512],
    keys_released: [bool; 512],

    mouse_position: Vec2,
    mouse_buttons_down: [bool; 8],
    mouse_buttons_pressed: [bool; 8],
    mouse_buttons_released: [bool; 8],
    mouse_wheel: f32,

    previous_keys:[bool; 512],
    previous_mouse_buttons: [bool; 8]
}

/// Implementation for engine
impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: [false; 512],
            keys_pressed: [false; 512],
            keys_released: [false; 512],
            mouse_position: Vec2::ZERO,
            mouse_buttons_down: [false; 8],
            mouse_buttons_pressed: [false; 8],
            mouse_buttons_released: [false; 8],
            mouse_wheel: 0.0,
            previous_keys: [false; 512],
            previous_mouse_buttons: [false; 8],
        }
    }

    pub fn new_frame(&mut self) {
        // Copy current state to previous for change detection
        self.previous_keys = self.keys_down;
        self.previous_mouse_buttons = self.mouse_buttons_down;
        
        // Clear one-frame states
        self.keys_pressed.fill(false);
        self.keys_released.fill(false);
        self.mouse_buttons_pressed.fill(false);
        self.mouse_buttons_released.fill(false);
        self.mouse_wheel = 0.0;
    }

    pub fn handle_key_down(&mut self, key: sapp::Keycode) {        
        let key_idx = key as usize;
        if key_idx < self.keys_down.len() {
            if !self.previous_keys[key_idx] && !self.keys_down[key_idx] {
                self.keys_pressed[key_idx] = true;
            }
            self.keys_down[key_idx] = true;
        }
    }

    pub fn handle_key_up(&mut self, key: sapp::Keycode) {
        let key_idx = key as usize;
        if key_idx < self.keys_down.len() {
            if self.keys_down[key_idx] {
                self.keys_released[key_idx] = true;
            }
            self.keys_down[key_idx] = false;
        }
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
    }

    pub fn handle_mouse_button_down(&mut self, button: sapp::Mousebutton) {
        let btn_idx = button as usize;
        if btn_idx < self.mouse_buttons_down.len() {
            if !self.previous_mouse_buttons[btn_idx] && !self.mouse_buttons_down[btn_idx] {
                self.mouse_buttons_pressed[btn_idx] = true;
            }
            self.mouse_buttons_down[btn_idx] = true;
        }
    }

    pub fn handle_mouse_button_up(&mut self, button: sapp::Mousebutton) {
        let btn_idx = button as usize;
        if btn_idx < self.mouse_buttons_down.len() {
            if self.mouse_buttons_down[btn_idx] {
                self.mouse_buttons_released[btn_idx] = true;
            }
            self.mouse_buttons_down[btn_idx] = false;
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: f32) {
        self.mouse_wheel += delta; // Accumulate wheel movement this frame
    }
}

/// Public functions for Game interface
impl InputManager {
    pub fn is_key_down(&self, key: sapp::Keycode) -> bool {
        let key_idx = key as usize;
        key_idx < self.keys_down.len() && self.keys_down[key_idx]
    }

    pub fn is_key_pressed(&self, key: sapp::Keycode) -> bool {
        let key_idx = key as usize;
        key_idx < self.keys_pressed.len() && self.keys_pressed[key_idx]
    }

    pub fn is_key_released(&self, key: sapp::Keycode) -> bool {
        let key_idx = key as usize;
        key_idx < self.keys_released.len() && self.keys_released[key_idx]
    }

    // Mouse queries
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn is_mouse_button_down(&self, button: sapp::Mousebutton) -> bool {
        let btn_idx = button as usize;
        btn_idx < self.mouse_buttons_down.len() && self.mouse_buttons_down[btn_idx]
    }

    pub fn is_mouse_button_pressed(&self, button: sapp::Mousebutton) -> bool {
        let btn_idx = button as usize;
        btn_idx < self.mouse_buttons_pressed.len() && self.mouse_buttons_pressed[btn_idx]
    }

    pub fn is_mouse_button_released(&self, button: sapp::Mousebutton) -> bool {
        let btn_idx = button as usize;
        btn_idx < self.mouse_buttons_released.len() && self.mouse_buttons_released[btn_idx]
    }

    pub fn mouse_wheel_delta(&self) -> f32 {
        self.mouse_wheel
    }
}