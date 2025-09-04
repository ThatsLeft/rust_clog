use std::hash::{DefaultHasher, Hash, Hasher};

use glam::{Vec2, Vec4};
use sokol::{app as sapp, gfx as sg};
use crate::engine::{check_collision, check_collision_with_point, AnimationManager, Camera2D, Circle, Collider, Game, GameConfig, InputManager, LoopType::Loop, ParticleSystem, Quad, Renderer, Sprite, SpriteAnimations};

pub struct TestGame {
    frame_count: u64,
    current_background: sg::Color,
    my_box: Quad,
    my_circle: Circle,
    world_boxes: Vec<Quad>,
    test_sprite: Sprite,
    texture_names: Vec<String>,
}

// Functions and functionality for the test game
impl TestGame {
    pub fn new() -> Self {
        let mut world_boxes = Vec::new();
        for i in 0..20 {
            
            let mut hasher = DefaultHasher::new();
            (i as u64).hash(&mut hasher);
            let seed = hasher.finish();
            
            let x = ((seed & 0xFFFF) as f32 / 65535.0) * 2000.0 - 1000.0;  // Random from -1000 to 1000
            let y = (((seed >> 16) & 0xFFFF) as f32 / 65535.0) * 1500.0 - 750.0;  // Random from -750 to 750
            
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
            my_box: Quad::new(-500.0, 0.0, 100.0, 50.0, Vec4::new(1.0, 0.0, 0.0, 1.0)), // Start at origin
            my_circle: Circle::new(200.0, 200.0, 75.0, Vec4::new(0.0, 1.0, 0.0, 1.0)),
            world_boxes,
            test_sprite: Sprite::new()  // ADD THIS - starts as solid color
                .with_position(Vec2::new(-10.0, -100.0))
                .with_size(Vec2::new(32.0, 32.0))
                .with_color(Vec4::new(1.0, 0.5, 0.8, 1.0))
                .with_texture_name("player".to_string())
                .with_flip_y(true),
            texture_names: vec![
                "player".to_string(),
                "bullet".to_string(), 
                "alien".to_string(),
                "ship_thruster_spritesheet".to_string()
            ],
        }
    }

    fn update_background_color(&mut self) {
        let g = self.current_background.g + 0.01;
        self.current_background.g = if g > 1.0 { 0.0 } else { g };
    }

    fn check_sprite_collisions(&self) -> Option<(usize, Vec2)> {
        let player_collider = self.get_player_collider();
        let single_box_colliders = self.get_single_box_collider();

        // Check collision with all world boxes
        for (index, box_collider) in self.get_box_colliders().iter().enumerate() {
            let result = check_collision_with_point(&player_collider, box_collider);
            if result.collided {
                return Some((index, result.contact_point));
            }
        }

        if check_collision(&player_collider, &single_box_colliders) {
            println!("Hit my box");
            return None;
        }

        // Check collision with circle
        let circle_collider = Collider::new_circle(
            self.my_circle.center.x, 
            self.my_circle.center.y, 
            self.my_circle.radius
        );
        
        if check_collision(&player_collider, &circle_collider) {
            println!("Hit circle");
            return None;
        }
        
        None
    }

    fn get_player_collider(&self) -> Collider {
        // Sprite position is already in world coordinates
        Collider::new_rect(
            self.test_sprite.position.x - self.test_sprite.size.x / 2.0,
            self.test_sprite.position.y - self.test_sprite.size.y / 2.0,
            self.test_sprite.size.x,
            self.test_sprite.size.y
        )
    }

    fn get_single_box_collider(&self) -> Collider {
        Collider::new_rect(
                self.my_box.position.x,
                self.my_box.position.y,
                self.my_box.size.x,
                self.my_box.size.y,
            )
    }
    
    fn get_box_colliders(&self) -> Vec<Collider> {
        self.world_boxes.iter().map(|quad| {
            Collider::new_rect(
                quad.position.x,
                quad.position.y, 
                quad.size.x, 
                quad.size.y
            )
        }).collect()
    }
}

/// Implement the Game aspect for the TestGame
/// init the game
/// update the game loop
/// render the game objects
/// handle_event handle events from the window not related to movement
impl Game for TestGame {
    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("My Awesome Test Game")
            .with_size(1000, 800)
            .with_background(sg::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 })
            .with_samples(4)
            .with_high_dpi(true)
    }
    
    fn init(&mut self, config: &GameConfig, renderer: &mut Renderer, animation_manager: &mut AnimationManager) {

        self.current_background = config.background_color;
        
        // Load the texture once here.
        for texture_name in &self.texture_names {
            let path = format!("assets/{}.png", texture_name);
            if let Ok(_) = renderer.load_texture(texture_name, &path) {
                println!("Loaded texture: {}", texture_name);
            } else {
                eprintln!("Failed to load texture: {}", texture_name);
            }
        }

        animation_manager.register_animation(SpriteAnimations::new(
            "player_thruster".to_string(), 
            "ship_thruster_spritesheet".to_string(), 
            Vec2::new(32.0, 32.0), 
            4, 
            4, 
            1.0, 
            Loop,
        )
            // {
            // name: "player_thruster".to_string(),
            // texture_name: "ship_thruster_spritesheet".to_string(),
            // frame_size: Vec2::new(32.0, 32.0),  // Assuming 32x32 frames
            // frame_count: 4,
            // frames_per_row: 4,
            // duration: 1.0,  // 1 second for full animation
            // looping: true,
            // }
        );

        println!("Game initialized!");
        println!("Window size: {}x{}", sapp::width(), sapp::height());

    }

    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D, animation_manager: &mut AnimationManager, particle_systems: &mut Vec<ParticleSystem>) {
        self.frame_count += 1;
        let time: f32 = self.frame_count as f32 * dt;
        
        self.update_background_color();

        self.my_circle.radius = 75.0 + (time * 3.0).sin() * 25.0;


        let old_position = self.test_sprite.position;

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

        if box_movement.length() > 0.0 {
            let movement = box_movement.normalize() * 200.0 * dt;
            // let old_position = self.test_sprite.position;
            self.test_sprite.position += movement;
        
            // Calculate rotation
            self.test_sprite.rotation = box_movement.y.atan2(box_movement.x) - std::f32::consts::PI / 2.0;
            
            // Check for collisions
            if let Some((hit_box_index, collision_point)) = self.check_sprite_collisions() {
                let explosion = ParticleSystem::new(collision_point, 80.0, 0.2, 0.5); // 50 particles per second
                camera.add_shake(5.0, 0.2);
                particle_systems.push(explosion);

                // Remove the hit box
                self.world_boxes.remove(hit_box_index);
                println!("Box collected! {} boxes remaining", self.world_boxes.len());
                
                // Don't revert position - allow player to move through the collected box
            }
        }
        
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
        if input.is_mouse_button_pressed(sapp::Mousebutton::Left) {
            let mouse_world_pos = camera.screen_to_world(input.mouse_position());
            println!("Clicked world position: ({:.1}, {:.1})", mouse_world_pos.x, mouse_world_pos.y);
            
            // Use the world position for distance calculation
            let distance = (mouse_world_pos - self.my_circle.center).length();
            if distance <= self.my_circle.radius {
                println!("Change color {:.1}", self.my_circle.center);
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
            self.test_sprite.rotation += (-dt * 1.0);
        }
        if input.is_key_down(sapp::Keycode::T) {
            camera.rotate_by(dt * 1.0);
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
        
        // sprite changing
        if input.is_key_pressed(sapp::Keycode::Enter) {

            let next_texture_id = self.texture_names.iter().position(|s| s == &self.test_sprite.texture_name).unwrap_or(0);
            let next_texture = self.texture_names[(next_texture_id+1)% self.texture_names.len()].clone();

            // Start animation if switching to player
            self.test_sprite.change_texture(next_texture);

            if self.test_sprite.texture_name == "ship_thruster_spritesheet" {
                self.test_sprite.flip_y = false;
                animation_manager.play_animation(&mut self.test_sprite, "player_thruster");
            } else {
                animation_manager.stop_animation(&mut self.test_sprite);
                animation_manager.clear_animation(&mut self.test_sprite);

                self.test_sprite.size = Vec2::new(32.0, 32.0);  // ADD THIS
                self.test_sprite.uv = Vec4::new(0.0, 0.0, 1.0, 1.0); 
            }
            
            println!("Switched to texture: {}", self.test_sprite.texture_name);
            
        }

        //animation
        animation_manager.update_sprite_animation(&mut self.test_sprite, dt);

        if self.frame_count % 60 == 0 {
            println!("FPS: {:.1} | Camera: pos({:.0}, {:.0}) zoom({:.2}) rot({:.2})", 
                     1.0 / dt, 
                     camera.get_position().x, camera.get_position().y,
                     camera.get_zoom(), camera.get_rotation());
        }
    }

    fn render(&mut self, renderer: &mut Renderer, camera: &mut Camera2D, particle_systems: &mut Vec<ParticleSystem>) {
        // Game responsibility: Decide what to draw
        
        renderer.begin_frame();

        for world_box in &self.world_boxes {
            renderer.draw_quad(world_box);
        }

        renderer.draw_quad(&self.my_box);
        renderer.draw_circle(&self.my_circle);
        
        renderer.draw_sprite(&self.test_sprite);
        
        for system in particle_systems {
            for particle in &system.particles {
                renderer.draw_particle(particle);
            }
        }
    }

    // handle events that are not movement based
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