use glam::{Vec2, Vec4};
use rand::Rng;
use sokol::{app::{self as sapp}, gfx as sg};
use crate::engine::{check_collision, check_collision_with_point, AnimationManager, Camera2D, Circle, Collider, CollisionShape, Game, GameConfig, InputManager, LoopType::{self}, ParticleSystem, Quad, Renderer, Sprite, SpriteAnimations, SystemState};

/// The current games statemachine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestGameState {
    MainMenu,
    Playing,
    Paused,
    Loading,
    Dead,
}

impl crate::engine::GameState for TestGameState {
    fn default_active() -> Self {
        Self::MainMenu
    }
}

pub struct TestGame {
    frame_count: u64,
    current_background: sg::Color,
    new_background: bool,
    asteroids: Vec<Circle>,
    player: Sprite,
    texture_names: Vec<String>,
    game_state: TestGameState,
    requested_system_state: Option<SystemState>,
}

// Functions and functionality for the test game
impl TestGame {
    pub fn new() -> Self {
        let mut asteroids = Vec::new();
        let mut rng = rand::rng();

        for i in 0..20 {
            let x = rng.random_range(-1000.0..=1000.0);
            let y = rng.random_range(-750.0..=750.0);
            
            let color = match i % 4 {
                0 => Vec4::new(1.0, 0.5, 0.0, 1.0), // Orange
                1 => Vec4::new(0.5, 0.0, 1.0, 1.0), // Purple  
                2 => Vec4::new(0.0, 1.0, 1.0, 1.0), // Cyan
                _ => Vec4::new(1.0, 1.0, 0.0, 1.0), // Yellow
            };

            let radius = rng.random_range(5.0..=75.0);
            let segments = rng.random_range(5.0..=32.0) as u32;
            
            asteroids.push(Circle::new(x, y, radius, color).with_segments(segments));
        }

        Self {
            frame_count: 0,
            current_background: sg::Color { r: 0.0, g: 0.4, b: 0.7, a: 1.0 },
            new_background: false,
            asteroids,
            player: Sprite::new()  // ADD THIS - starts as solid color
                .with_position(Vec2::new(-10.0, -100.0))
                .with_size(Vec2::new(64.0, 64.0))
                .with_color(Vec4::new(1.0, 0.5, 0.8, 1.0))
                .with_texture_name("ship".to_string()),
            texture_names: vec![
                "ship".to_string(),
                "bullet".to_string(), 
                "alien".to_string(),
                "ship_thruster_spritesheet".to_string(),
                "alien_movement".to_string()
            ],
            game_state: TestGameState::Loading,
            requested_system_state: None,
        }
    }

    fn update_background_color(&mut self) {
        let g = self.current_background.g + 0.01;
        self.current_background.g = if g > 1.0 { 0.0 } else { g };
        self.new_background = true;
    }

    fn check_sprite_collisions(&self) -> Option<(usize, Vec2)> {
        let player_collider = self.get_player_collider();

        // Check collision with all world boxes
        for (index, box_collider) in self.get_astroid_colliders().iter().enumerate() {
            let result = check_collision_with_point(&player_collider, box_collider);
            if result.collided {
                return Some((index, result.contact_point));
            }
        }
        
        None
    }

    fn get_player_collider(&self) -> Collider {
        // Sprite position is already in world coordinates
        Collider::new_rect(
            self.player.position.x - self.player.size.x / 4.0,
            self.player.position.y - self.player.size.y / 4.0,
            self.player.size.x/2.0,
            self.player.size.y/2.0 
        )
    }
    
    fn get_astroid_colliders(&self) -> Vec<Collider> {
        self.asteroids.iter().map(|quad| {
            Collider::new_circle(
                quad.center.x,
                quad.center.y, 
                quad.radius,
            )
        }).collect()
    }

    fn reset_game(&mut self) {
        // Reset game to initial state
        self.frame_count = 0;
        self.current_background = sg::Color { r: 0.0, g: 0.4, b: 0.7, a: 1.0 };
        self.player.position = Vec2::new(-10.0, -100.0);
        
        // Regenerate world boxes
        let mut rng = rand::rng();
        self.asteroids.clear();
        
        for i in 0..20 {
            let x = rng.random_range(-1000.0..=1000.0);
            let y = rng.random_range(-750.0..=750.0);
            
            let color = match i % 4 {
                0 => Vec4::new(1.0, 0.5, 0.0, 1.0),
                1 => Vec4::new(0.5, 0.0, 1.0, 1.0),
                2 => Vec4::new(0.0, 1.0, 1.0, 1.0),
                _ => Vec4::new(1.0, 1.0, 0.0, 1.0),
            };
            
            let radius = rng.random_range(5.0..=75.0);
            let segments = rng.random_range(5.0..=32.0) as u32; 
            self.asteroids.push(Circle::new(x, y, radius, color).with_segments(segments));
        }
    }
}

/// Implement the Game aspect for the TestGame
/// init the game
/// update the game loop
/// render the game objects
/// handle_event handle events from the window not related to movement
impl Game for TestGame {
    type State = TestGameState;

    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("My Awesome Test Game")
            .with_size(1000, 800)
            .with_background(sg::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 })
            .with_samples(4)
            .with_high_dpi(false)
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
            LoopType::Loop,
        ));

        animation_manager.register_animation(SpriteAnimations::new(
            "squid_idle".to_string(), 
            "squid".to_string(), 
            Vec2::new(32.0, 32.0), 
            8, 
            8, 
            1.0, 
            LoopType::PingPong,
        ));

        println!("Game initialized!");
        println!("Window size: {}x{}", sapp::width(), sapp::height());
        self.game_state = TestGameState::MainMenu;

    }

    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D, animation_manager: &mut AnimationManager, particle_systems: &mut Vec<ParticleSystem>) {
        // Handle game state transitions first
        match self.game_state {
            TestGameState::MainMenu => {
                if input.is_key_pressed(sapp::Keycode::Enter) {
                    self.game_state = TestGameState::Playing;
                    println!("Starting game!");
                }
                if input.is_key_pressed(sapp::Keycode::Escape) {
                    self.requested_system_state = Some(SystemState::Shutdown);
                }
            }
            TestGameState::Playing => {

                self.frame_count += 1;
                let time: f32 = self.frame_count as f32 * dt;
        
                if input.is_key_pressed(sapp::Keycode::P) {
                    self.game_state = TestGameState::Paused;
                }
                if input.is_key_pressed(sapp::Keycode::M) {
                    self.game_state = TestGameState::MainMenu;
                }
                // Continue with existing game logic below

                let mut player_movement = Vec2::ZERO;
                if input.is_key_down(sapp::Keycode::W) {
                    player_movement.y += 1.0;
                }
                if input.is_key_down(sapp::Keycode::S) {
                    player_movement.y -= 1.0;
                }
                if input.is_key_down(sapp::Keycode::A) {
                    player_movement.x -= 1.0;
                }
                if input.is_key_down(sapp::Keycode::D) {
                    player_movement.x += 1.0;
                }

                if player_movement.length() > 0.0 {
                    let movement = player_movement.normalize() * 200.0 * dt;
                    // let old_position = self.test_sprite.position;
                    self.player.position += movement;
                
                    // Calculate rotation
                    self.player.rotation = player_movement.y.atan2(player_movement.x) - std::f32::consts::PI / 2.0;
                    
                    // Check for collisions
                    if let Some((hit_box_index, collision_point)) = self.check_sprite_collisions() {
                        let explosion = ParticleSystem::new(collision_point, 80.0, 0.2, 0.5);
                        camera.add_shake(5.0, 0.2);
                        particle_systems.push(explosion);

                        // Remove the hit box
                        self.asteroids.remove(hit_box_index);
                        println!("Box collected! {} boxes remaining", self.asteroids.len());
                    }
                }

                // Camera follows the box with some offset
                let target_camera_pos = self.player.position + Vec2::new(50.0, 25.0);
                camera.set_position(target_camera_pos);
            }
            TestGameState::Paused => {
                if input.is_key_pressed(sapp::Keycode::P) {
                    self.game_state = TestGameState::Playing;
                }
                if input.is_key_pressed(sapp::Keycode::M) {
                    self.game_state = TestGameState::MainMenu;
                }
                if input.is_key_pressed(sapp::Keycode::Escape) {
                    self.requested_system_state = Some(SystemState::Shutdown);
                }
            }
            TestGameState::Dead => {
                if input.is_key_pressed(sapp::Keycode::Enter) {
                    self.reset_game();
                }
                if input.is_key_pressed(sapp::Keycode::Escape) {
                    self.game_state = TestGameState::MainMenu;
                }
            }
            TestGameState::Loading => {
                // Handle loading/transition logic
            }
        }
        // self.update_background_color();
        
    }

    fn render(&mut self, renderer: &mut Renderer, camera: &mut Camera2D, particle_systems: &mut Vec<ParticleSystem>) {
        renderer.begin_frame();
    
        match self.game_state {
            TestGameState::MainMenu => {
                // Render main menu
                let menu_bg = Quad::new(-400.0, -300.0, 800.0, 600.0, Vec4::new(0.1, 0.1, 0.3, 0.9));
                renderer.draw_quad(&menu_bg);
                
                let title_box = Quad::new(-150.0, 100.0, 300.0, 80.0, Vec4::new(0.8, 0.8, 0.0, 1.0));
                renderer.draw_quad(&title_box);
                
                let start_button = Quad::new(-100.0, 0.0, 200.0, 50.0, Vec4::new(0.0, 0.8, 0.0, 1.0));
                renderer.draw_quad(&start_button);
            }
            TestGameState::Playing => {
                // Your existing game rendering
                for astroid in &self.asteroids {
                    renderer.draw_circle(astroid);
                }
                renderer.draw_sprite(&self.player);

                let player_collider = &self.get_player_collider();
                let (width, height) = match player_collider.shape {
                    CollisionShape::Rectangle { width, height } => (width, height),
                    CollisionShape::Circle { radius } => (radius * 2.0, radius * 2.0), // Use diameter for both dimensions
                };
                
                let player_collider_outline = Quad::new(
                    player_collider.position.x, 
                    player_collider.position.y, 
                    width, 
                    height, 
                    Vec4::new(1.0, 0.0, 0.0, 1.0)).with_outline();

                renderer.draw_quad(&player_collider_outline);
                
                for system in particle_systems {
                    for particle in &system.particles {
                        renderer.draw_particle(particle);
                    }
                }
            }
            TestGameState::Paused => {
                // Render game in background + pause overlay
                for astroid in &self.asteroids {
                    renderer.draw_circle(astroid);
                }
                renderer.draw_sprite(&self.player);
                
                for system in particle_systems {
                    for particle in &system.particles {
                        renderer.draw_particle(particle);
                    }
                }
                
                // Pause overlay
                let pause_overlay = Quad::new(
                    camera.get_position().x - 400.0,
                    camera.get_position().y - 300.0,
                    800.0, 600.0,
                    Vec4::new(0.0, 0.0, 0.0, 0.5)
                );
                renderer.draw_quad(&pause_overlay);
                
                let pause_text = Quad::new(
                    camera.get_position().x - 100.0,
                    camera.get_position().y,
                    200.0, 50.0,
                    Vec4::new(1.0, 1.0, 1.0, 1.0)
                );
                renderer.draw_quad(&pause_text);
            }
            _ => {
                // Handle other states as needed
            }
        }
    }

    // handle events that are not movement based
    fn handle_event(&mut self, event: &sapp::Event) {
        match event._type {
            sapp::EventType::KeyDown => {
                match event.key_code {
                    sapp::Keycode::F1 => {
                        println!("F1, toggle debug information!");
                    }
                    _ => {}
                }
            }
            sapp::EventType::Unfocused => {
                println!("Game suspended!");
                self.game_state = TestGameState::Paused;
                self.requested_system_state = Some(SystemState::Background);
            }
            sapp::EventType::Focused => {
                println!("Game active!");
                self.requested_system_state = Some(SystemState::GameActive);
            }
            _ => {}
        }
    }

    fn request_background_color_change(&self) -> Option<sg::Color> {
        if self.new_background {
            return Some(self.current_background)
        }
        None
    }

    fn request_system_state(&mut self) -> Option<SystemState> {
        let state = self.requested_system_state;
        self.requested_system_state = None; // Clear after returning
        state
    }
    
    fn engine_render_loading(&mut self, renderer: &mut Renderer, progress: f32) {
        let loading_box = Quad::new(-200.0, -50.0, 400.0 * progress, 20.0, 
                                  Vec4::new(0.0, 1.0, 0.0, 1.0));
        renderer.draw_quad(&loading_box);
    }
    
}