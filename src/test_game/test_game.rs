use std::hash::{DefaultHasher, Hash, Hasher};

use glam::{Vec2, Vec4};
use sokol::{app as sapp, gfx as sg};
use crate::engine::{Camera2D, Circle, Game, GameConfig, InputManager, Quad, Renderer};

pub struct TestGame {
    frame_count: u64,
    current_background: sg::Color,
    my_box: Quad,
    my_circle: Circle,
    world_boxes: Vec<Quad>,
}

impl TestGame {
    pub fn new() -> Self {
        let mut world_boxes = Vec::new();
        for i in 0..20 {
            let x = (i as f32 * 200.0) % 2000.0 - 1000.0; // Spread from -1000 to 1000
            let y = (i as f32 * 150.0) % 1500.0 - 750.0;  // Spread from -750 to 750
            let color = match i % 4 {
                0 => Vec4::new(1.0, 0.5, 0.0, 1.0), // Orange
                1 => Vec4::new(0.5, 0.0, 1.0, 1.0), // Purple  
                2 => Vec4::new(0.0, 1.0, 1.0, 1.0), // Cyan
                _ => Vec4::new(1.0, 1.0, 0.0, 1.0), // Yellow
            };
            world_boxes.push(Quad::new(x, y, 80.0, 60.0, color));
        }

        Self {
            frame_count: 0,
            current_background: sg::Color { r: 0.0, g: 0.4, b: 0.7, a: 1.0 },
            my_box: Quad::new(0.0, 0.0, 100.0, 50.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Start at origin
            my_circle: Circle::new(200.0, 200.0, 75.0, Vec4::new(0.0, 1.0, 0.0, 1.0)),
            world_boxes,
        }
    }

    fn update_background_color(&mut self) {
        let g = self.current_background.g + 0.01;
        self.current_background.g = if g > 1.0 { 0.0 } else { g };
    }
}

impl Game for TestGame {
    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("My Awesome Test Game")
            .with_size(1024, 768)
            .with_background(sg::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 })
            .with_samples(4)
    }
    
    fn init(&mut self, config: &GameConfig) {

        self.current_background = config.background_color;

        println!("Game initialized!");
        println!("Window size: {}x{}", sapp::width(), sapp::height());
    }

    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D) {
        self.frame_count += 1;
        
        // self.update_background_color();

        let time: f32 = self.frame_count as f32 * dt;

        self.my_circle.radius = 75.0 + (time * 3.0).sin() * 25.0;

        // test
        // Move box with WASD
        let mut box_movement = Vec2::ZERO;
        if input.is_key_down(sapp::Keycode::W) {
            box_movement.y += 1.0;
        }
        if input.is_key_down(sapp::Keycode::S) {
            box_movement.y -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::A) {
            box_movement.x -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::D) {
            box_movement.x += 1.0;
        }

        self.my_box.position += box_movement * 200.0 * dt;
        
        // Move circle with arrow keys (just for demonstration)
        let mut circle_movement = Vec2::ZERO;
        if input.is_key_down(sapp::Keycode::Up) {
            circle_movement.y += 1.0;
        }
        if input.is_key_down(sapp::Keycode::Down) {
            circle_movement.y -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::Left) {
            circle_movement.x -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::Right) {
            circle_movement.x += 1.0;
        }
        
        if circle_movement.length() > 0.0 {
            circle_movement = circle_movement.normalize() * 150.0 * dt;
            self.my_circle.center += circle_movement;
        }

        // Change colors with number keys (one-time actions)
        if input.is_key_pressed(sapp::Keycode::Num1) {
            self.my_box.color = Vec4::new(1.0, 0.0, 0.0, 1.0); // Red
        }
        if input.is_key_pressed(sapp::Keycode::Num2) {
            self.my_box.color = Vec4::new(0.0, 1.0, 0.0, 1.0); // Green
        }
        if input.is_key_pressed(sapp::Keycode::Num3) {
            self.my_box.color = Vec4::new(0.0, 0.0, 1.0, 1.0); // Blue
        }
        
        // Mouse interaction - change circle color when clicked
        if input.is_mouse_button_down(sapp::Mousebutton::Left) {
            let mouse_pos = input.mouse_position();
            let distance = (mouse_pos - self.my_circle.center).length();
            if distance <= self.my_circle.radius {
                // Clicked inside circle
                let mut hasher = DefaultHasher::new();
                self.frame_count.hash(&mut hasher);
                let hash = hasher.finish();
                self.my_circle.color = Vec4::new(
                    ((hash & 0xFF) as f32) / 255.0,
                    (((hash >> 8) & 0xFF) as f32) / 255.0,
                    (((hash >> 16) & 0xFF) as f32) / 255.0,
                    1.0
                );
            }
        }

        // Example camera controls:
        
        // Camera follows the box with some offset
        // let target_camera_pos = self.my_box.position + Vec2::new(50.0, 25.0);
        // camera.set_position(target_camera_pos);
        let mut camera_movement = Vec2::ZERO;
        if input.is_key_down(sapp::Keycode::I) {
            camera_movement.y += 1.0;
        }
        if input.is_key_down(sapp::Keycode::K) {
            camera_movement.y -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::J) {
            camera_movement.x -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::L) {
            camera_movement.x += 1.0;
        }

        if camera_movement.length() > 0.0 {
            camera_movement = camera_movement.normalize() * 300.0 * dt;
            camera.move_by(camera_movement);
        }
        
        // Zoom control with Q/E keys
        if input.is_key_down(sapp::Keycode::Q) {
            camera.zoom_by(-dt * 1.0); // Zoom out
        }
        if input.is_key_down(sapp::Keycode::E) {
            camera.zoom_by(dt * 1.0);   // Zoom in  
        }
        
        // Camera rotation with R/T keys (for fun)
        if input.is_key_down(sapp::Keycode::R) {
            camera.rotate_by(dt * 1.0);
        }
        if input.is_key_down(sapp::Keycode::T) {
            camera.rotate_by(-dt * 1.0);
        }
        
        // Reset camera with spacebar
        if input.is_key_pressed(sapp::Keycode::Space) {
            camera.set_position(Vec2::ZERO);
            camera.set_zoom(1.0);
            camera.set_rotation(0.0);
        }

        if input.is_key_pressed(sapp::Keycode::Num4) {
            camera.add_shake(10.0, 0.5); // Medium shake for 0.5 seconds
        }
        if input.is_key_pressed(sapp::Keycode::Num5) {
            camera.add_shake(20.0, 1.0); // Strong shake for 1 second
        }
        
        // Mouse interaction using camera coordinate conversion
        if input.is_mouse_button_pressed(sapp::Mousebutton::Left) {
            let mouse_world_pos = camera.screen_to_world(input.mouse_position());
            println!("Clicked world position: ({:.1}, {:.1})", mouse_world_pos.x, mouse_world_pos.y);
            
            // Check if clicked on circle (now works correctly with camera transform)
            let distance = (mouse_world_pos - self.my_circle.center).length();
            if distance <= self.my_circle.radius {
                println!("Clicked on circle!");
                // Change circle color logic here
            }
        }
        
        if self.frame_count % 60 == 0 {
            println!("FPS: {:.1} | Camera: pos({:.0}, {:.0}) zoom({:.2}) rot({:.2})", 
                     1.0 / dt, 
                     camera.get_position().x, camera.get_position().y,
                     camera.get_zoom(), camera.get_rotation());
        }
    }

    fn render(&mut self) {
        // Nothing to render yet - the pass_action in the engine handles clearing
        // Future sprite rendering will go here
    }

    fn render_with_renderer(&mut self, renderer: &mut Renderer, camera: &mut Camera2D) {
        // Game responsibility: Decide what to draw
        for world_box in &self.world_boxes {
            renderer.draw_quad(world_box);
        }

        renderer.draw_quad(&self.my_box);
        renderer.draw_circle(&self.my_circle);
    }

    fn handle_event(&mut self, event: &sapp::Event) {
        match event._type {
            sapp::EventType::KeyDown => {
                match event.key_code {
                    sapp::Keycode::Escape => {
                        println!("Escape pressed - quitting!");
                        sapp::request_quit();
                    }
                    _ => {}
                }
            }
            sapp::EventType::Resized => {
                println!("Window resized to: {}x{}", event.window_width, event.window_height);
            }
            _ => {}
        }
    }

    fn get_background_color(&self) -> Option<sg::Color> {
        Some(self.current_background)
    }


}