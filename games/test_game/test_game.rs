use glam::{Vec2, Vec3, Vec4};
use rand::Rng;
use sokol::{app::{self as sapp}, gfx as sg};
use crate::engine::{check_collision, check_collision_with_point, AnimationManager, Camera2D, Circle, Collider, CollisionShape, Game, GameConfig, InputManager, LoopType::{self}, ParticleSystem, Quad, Renderer, Sprite, SpriteAnimations, SystemState};
use std::{collections::HashMap, string};

/// The current games statemachine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestGameState {
    MainMenu,
    Playing,
    Paused,
    Loading,
    Dead,
    Completed,
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
    player_animation: bool,
    player_thruster_idx: Option<String>,
    texture_names: Vec<String>,
    game_state: TestGameState,
    requested_system_state: Option<SystemState>,
    world_min: Vec2,
    world_max: Vec2,
    completed_fx_started: bool,
    completed_fx_timer: f32,
    completed_fx_next_burst: f32,
    text: Option<crate::engine::TextRenderer>,
    hud_msg: Option<String>,
    hud_timer: f32,
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
            player_animation: false,
            player_thruster_idx: None,
            texture_names: vec![
                "ship".to_string(),
                "bullet".to_string(), 
                "alien".to_string(),
                "ship_thruster_spritesheet".to_string(),
                "alien_movement".to_string()
            ],
            game_state: TestGameState::Loading,
            requested_system_state: None,
            world_min: Vec2::new(-1000.0, -750.0),
            world_max: Vec2::new(1000.0, 750.0),
            completed_fx_started: false,
            completed_fx_timer: 0.0,
            completed_fx_next_burst: 0.0,
            text: None,
            hud_msg: None,
            hud_timer: 0.0,
        }
    }

    fn update_background_color(&mut self, new_color:sg::Color) {
        self.current_background = new_color;
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

    fn reset_game(&mut self, camera: &mut Camera2D) {
        // Reset game to initial state
        self.frame_count = 0;
        self.current_background = sg::Color { r: 0.0, g: 0.4, b: 0.7, a: 1.0 };
        self.player.position = Vec2::new(-10.0, -100.0);
        self.game_state = TestGameState::Playing;
        
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

        let target_camera_pos = self.player.position + Vec2::new(50.0, 25.0);
        camera.set_position(target_camera_pos);
    }

    fn spawn_firework_burst(&self, center: Vec2, count: usize, speed: f32, life: f32, color: Vec4, particle_systems: &mut HashMap<String, ParticleSystem>) {
        // Emit ~count particles quickly (single burst)
        let burst_duration = 0.05f32.max(0.001);
        let emission_rate = (count as f32 / burst_duration).max(1.0);

        let sys = ParticleSystem::new(center, emission_rate, burst_duration, life)
            .with_velocity_radial(speed * 0.7, speed * 1.2)
            .with_fixed_color(color)
            .with_drag(0.0);

        let key = format!("firework_{:.2}_{:.2}_{}", center.x, center.y, rand::rng().random_range(0..1_000_000));
        particle_systems.insert(key, sys);
    }

    fn spawn_confetti_rain(&self, area_min: Vec2, area_max: Vec2, duration: f32, particle_systems: &mut HashMap<String, ParticleSystem>) {
        // Steady rain from the top edge across the width
        let width = area_max.x - area_min.x;
        let top_y = area_max.y;

        // Pick a center spawn; system will randomize velocity, we randomize color via palette
        let sys = ParticleSystem::new(Vec2::new(area_min.x + width * 0.5, top_y), 120.0, duration, 1.2)
            .with_velocity_range(Vec2::new(-20.0, -160.0), Vec2::new(20.0, -100.0))
            .with_color_palette(vec![
                Vec4::new(1.0, 0.2, 0.2, 1.0),
                Vec4::new(1.0, 0.6, 0.2, 1.0),
                Vec4::new(1.0, 1.0, 0.2, 1.0),
                Vec4::new(0.2, 0.9, 0.4, 1.0),
                Vec4::new(0.3, 0.6, 1.0, 1.0),
                Vec4::new(0.8, 0.4, 1.0, 1.0),
            ])
            .with_drag(0.0);

        let key = format!("confetti_{}", rand::rng().random_range(0..1_000_000));
        particle_systems.insert(key, sys);
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
            .with_background(sg::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.8 })
            .with_samples(4)
            .with_high_dpi(false)
    }
    
    fn init(&mut self, config: &GameConfig, renderer: &mut Renderer, animation_manager: &mut AnimationManager, particle_systems: &mut HashMap<String, ParticleSystem>) {

        self.current_background = config.background_color;
        self.new_background = true;

        // Load the texture once here.
        for texture_name in &self.texture_names {
            let path = format!("assets/{}.png", texture_name);
            if let Ok(_) = renderer.load_texture(texture_name, &path) {
                println!("Loaded texture: {}", texture_name);
            } else {
                eprintln!("Failed to load texture: {}", texture_name);
            }
        }

        
        // Load font texture once
        let _ = renderer.load_texture("font", "assets/font.png");
        // Create a text renderer: 16x16 atlas, 8x8 glyphs (adjust to your atlas)
        self.text = Some(crate::engine::TextRenderer::new("font", 16.0, 16.0, 16, 6));
        

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

        let thruster = ParticleSystem::new(self.player.position, 120.0, f32::MAX, 0.7)
            .with_velocity_direction(glam::Vec2::new(0.0, 0.0), 00.0, 0.0, 0.0)
            .with_color_range(glam::Vec4::new(0.6, 0.0, 0.0, 1.0), glam::Vec4::new(1.0, 1.0, 0.0, 1.0))
            .with_drag(0.4);
        let thruster_key = "player_thruster".to_string();
        particle_systems.insert(thruster_key.clone(), thruster);
        // store None for now; direct key access is preferable
        self.player_thruster_idx = Some("player_thruster".to_string());

        println!("Game initialized!");
        println!("Window size: {}x{}", sapp::width(), sapp::height());
        self.game_state = TestGameState::MainMenu;

    }

    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D, animation_manager: &mut AnimationManager, particle_systems: &mut HashMap<String, ParticleSystem>) {
        if self.hud_timer > 0.0 {
            self.hud_timer -= dt;
            if self.hud_timer <= 0.0 {
                self.hud_msg = None;
            }
        }

        // Handle game state transitions
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
                    if !self.player_animation {
                        self.player.change_texture("ship_thruster_spritesheet".to_string());
                        animation_manager.play_animation(&mut self.player, "player_thruster");
                        self.player_animation = true;
                    } 
                    let movement = player_movement.normalize() * 200.0 * dt;
                    // let old_position = self.test_sprite.position;
                    self.player.position += movement;
                    let half = self.player.size * 0.5;
                    self.player.position.x = self.player.position.x.clamp(self.world_min.x + half.x, self.world_max.x - half.x);
                    self.player.position.y = self.player.position.y.clamp(self.world_min.y + half.y, self.world_max.y - half.y);
                
                    // Calculate rotation
                    self.player.rotation = player_movement.y.atan2(player_movement.x) - std::f32::consts::PI / 2.0;

                    // Update thruster system to follow and emit opposite to movement
                    if let Some(key) = &self.player_thruster_idx {
                        if let Some(sys) = particle_systems.get_mut(key) {
                            // Position thruster slightly behind ship
                            let backward = player_movement.normalize();
                            let offset = Vec2::new(-backward.x, -backward.y) * (self.player.size.y * 0.4);
                            sys.set_spawn_position(self.player.position + offset);
                            // Emit opposite of movement
                            sys.set_velocity_direction(Vec2::new(-backward.x, -backward.y), 40.0, 90.0, 0.5);
                            // Make sure thruster is on while moving
                            sys.set_emission_rate(120.0);
                        }
                    }
                    
                    // Check for collisions
                    if let Some((hit_box_index, collision_point)) = self.check_sprite_collisions() {
                        let color = self.asteroids[hit_box_index].color;
                        camera.add_shake(5.0, 0.2);
                        let explosion_system = ParticleSystem::new(collision_point, 50.0, 0.2, 1.5).with_fixed_color(color);
                        let key = format!("explosion_{}", rand::rng().random_range(0..1_000_000));
                        particle_systems.insert(key, explosion_system);

                        // Remove the hit box
                        self.asteroids.remove(hit_box_index);
                        self.hud_msg = Some(format!("Box collected! {} boxes remaining", self.asteroids.len()));
                        self.hud_timer = 1.5;
                    }
                }
                else {
                    self.player.change_texture("ship".to_string());
                    self.player.size = Vec2::new(32.0, 32.0);
                    self.player.uv = Vec4::new(0.0, 0.0, 1.0, 1.0); 
                    self.player_animation = false;
                    animation_manager.stop_animation(&mut self.player);

                    // Idle: keep thruster system but stop emission
                    if let Some(key) = &self.player_thruster_idx {
                        if let Some(sys) = particle_systems.get_mut(key) {
                            sys.set_emission_rate(0.0);
                            sys.set_spawn_position(self.player.position);
                        }
                    }

                }
                
                animation_manager.update_sprite_animation(&mut self.player, dt);

                // Camera follows the box with some offset
                let target_camera_pos = self.player.position + Vec2::new(50.0, 25.0);
                camera.set_position(target_camera_pos);
                camera.clamp_to_bounds(self.world_min, self.world_max);
                   
                if self.asteroids.is_empty() {
                    self.game_state = TestGameState::Completed;
                }
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
                    self.reset_game(camera);
                }
                if input.is_key_pressed(sapp::Keycode::Escape) {
                    self.game_state = TestGameState::MainMenu;
                }
            }
            TestGameState::Completed => {
                // Center camera for the celebration
                camera.set_position(Vec2::ZERO);
            
                // Start sequence once
                if !self.completed_fx_started {
                    self.completed_fx_started = true;
                    self.completed_fx_timer = 0.0;
                    self.completed_fx_next_burst = 0.0;
            
                    // Initial shake and triple bursts at center
                    camera.add_shake(6.0, 0.4);
                    let colors = [
                        Vec4::new(1.0, 0.3, 0.3, 1.0),
                        Vec4::new(1.0, 0.8, 0.2, 1.0),
                        Vec4::new(0.3, 0.8, 1.0, 1.0),
                    ];
                    for (i, c) in colors.iter().enumerate() {
                        let speed = 180.0 + (i as f32) * 60.0;
                        let life = 1.0 + (i as f32) * 0.25;
                        self.spawn_firework_burst(Vec2::ZERO, 28, speed, life, *c, particle_systems);
                    }
            
                    // Short confetti rain across a visible area
                    self.spawn_confetti_rain(Vec2::new(-300.0, -200.0), Vec2::new(300.0, 200.0), 0.8, particle_systems);
                }
            
                // Run timed bursts for ~2.5s
                self.completed_fx_timer += dt;
                if self.completed_fx_timer >= self.completed_fx_next_burst && self.completed_fx_timer < 2.5 {
                    self.completed_fx_next_burst += 0.22;
            
                    let t = self.completed_fx_timer;
                    let radius = 120.0 + (t * 60.0);
                    let angle = t * 5.0;
                    let pos = Vec2::new(angle.cos() * radius, angle.sin() * radius);
            
                    // Cycle colors
                    let hue = ((t * 2.0) as i32 % 6) as i32;
                    let color = match hue {
                        0 => Vec4::new(1.0, 0.4, 0.4, 1.0),
                        1 => Vec4::new(1.0, 0.8, 0.3, 1.0),
                        2 => Vec4::new(0.8, 1.0, 0.4, 1.0),
                        3 => Vec4::new(0.4, 1.0, 0.8, 1.0),
                        4 => Vec4::new(0.4, 0.7, 1.0, 1.0),
                        _ => Vec4::new(0.8, 0.5, 1.0, 1.0),
                    };
            
                    // Staggered mini-bursts
                    self.spawn_firework_burst(pos, 20, 160.0, 1.0, color, particle_systems);
                    self.spawn_firework_burst(-pos * 0.5, 16, 140.0, 0.8, color, particle_systems);
            
                    // subtle micro-shake
                    camera.add_shake(2.0, 0.1);
                }
            
                if input.is_key_pressed(sapp::Keycode::M) {
                    self.game_state = TestGameState::MainMenu;
                    self.completed_fx_started = false;
                }
                if input.is_key_pressed(sapp::Keycode::Enter) {
                    self.reset_game(camera);
                    self.completed_fx_started = false;
                }
            }
            TestGameState::Loading => {
                // Handle loading/transition logic
            }
        }
                
    }

    fn render(&mut self, renderer: &mut Renderer, camera: &mut Camera2D) {
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

                if let Some(text) = &self.text {
                    let mut t = text.clone();
                    t.set_scale(3.0);
                    t.set_color(Vec4::new(1.0, 1.0, 0.2, 1.0));
                    // Test with simple characters first
                    t.draw_text_world(renderer, glam::Vec2::new(-140.0, 120.0), "ABC");
                    t.set_scale(2.0);
                    t.set_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
                    t.draw_text_world(renderer, glam::Vec2::new(-90.0, 10.0), "123");
                }
            }
            TestGameState::Playing => {

                //draw window bounds
                let world_size = self.world_max - self.world_min;
                let border = Quad::new(self.world_min.x, self.world_min.y, world_size.x, world_size.y, Vec4::new(0.1, 0.1, 0.3, 0.9)).with_outline();
                renderer.draw_quad(&border);

                for astroid in &self.asteroids {
                    renderer.draw_circle(astroid);
                }
                let astroid_colliders = self.get_astroid_colliders();
                for astroid_collider in astroid_colliders {
                    let (radius) = match astroid_collider.shape {
                        CollisionShape::Rectangle { width, height } => (width - height),
                        CollisionShape::Circle { radius } => (radius), // Use diameter for both dimensions
                    };

                    let astroid_collider_outline = Circle::new(
                        astroid_collider.position.x, 
                        astroid_collider.position.y, 
                        radius, 
                        Vec4::new(1.0, 0.0, 0.0, 1.0)).with_outline();
                    renderer.draw_circle(&astroid_collider_outline);
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

                if let (Some(text), Some(msg)) = (&self.text, &self.hud_msg) {
                    if self.hud_timer > 0.0 {
                        let mut t = text.clone();
                        t.set_scale(1.0);
                        t.set_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
                        t.draw_top_right(renderer, camera, Vec2::new(20.0, 20.0), msg);
                    }
                }
            }
            TestGameState::Paused => {
                // Render game in background + pause overlay
                for astroid in &self.asteroids {
                    renderer.draw_circle(astroid);
                }
                renderer.draw_sprite(&self.player);
                                
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
            TestGameState::Completed => {
                
                // Pause overlay
                let completed_overlay = Quad::new(
                    camera.get_position().x - 400.0,
                    camera.get_position().y - 300.0,
                    800.0, 600.0,
                    Vec4::new(0.0, 0.0, 0.0, 0.5)
                );
                renderer.draw_quad(&completed_overlay);
                
                let completed_text = Quad::new(
                    camera.get_position().x - 100.0,
                    camera.get_position().y,
                    200.0, 50.0,
                    Vec4::new(1.0, 1.0, 1.0, 1.0)
                );
                renderer.draw_quad(&completed_text);
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
    
    fn engine_render_loading(&mut self, renderer: &mut Renderer, progress: f32, camera: &mut Camera2D) {
        let loading_box = Quad::new(-200.0, -50.0, 400.0 * progress, 20.0, Vec4::new(0.0, 1.0, 0.0, 1.0));
        renderer.draw_quad(&loading_box);

        if let Some(text) = &self.text {
            let mut txt = text.clone();
            txt.set_scale(1.0);
            txt.set_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
            let pct = (progress * 100.0).round() as i32;
            // world-space anchor near the left edge of the bar
            txt.draw_text_world(renderer, glam::Vec2::new(-190.0, -45.0), &format!("Loading {}%", pct));
        }
    }
    
}