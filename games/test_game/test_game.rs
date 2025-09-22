use crate::engine::{
    Camera2D, Circle, Collider, Game, GameConfig, InputManager,
    LoopType::{self},
    ParticleSystem, Quad, Sprite, SpriteAnimations,
};
use glam::{Vec2, Vec4};
use rand::Rng;
use rusclog::engine::{
    gravity::{GravityFalloff, GravityField},
    physics_world::PhysicsWorld,
    rigid_body::{BodyId, RigidBody},
    EngineServices, Renderer,
};
use sokol::{
    app::{self as sapp},
    gfx as sg,
};
use std::collections::HashMap;

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

pub struct TestGame {
    frame_count: u64,
    current_background: sg::Color,
    new_background: bool,
    asteroids: HashMap<BodyId, Circle>,
    player: Sprite,
    player_animation: bool,
    player_thruster_idx: Option<String>,
    player_body_id: Option<BodyId>,
    texture_names: Vec<String>,
    game_state: TestGameState,
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
        let asteroids = HashMap::new();

        Self {
            frame_count: 0,
            current_background: sg::Color {
                r: 0.0,
                g: 0.4,
                b: 0.7,
                a: 1.0,
            },
            new_background: false,
            asteroids,
            player: Sprite::new() // ADD THIS - starts as solid color
                .with_position(Vec2::new(-10.0, -100.0))
                .with_size(Vec2::new(64.0, 64.0))
                .with_color(Vec4::new(1.0, 0.5, 0.8, 1.0))
                .with_texture_name("ship".to_string()),
            player_animation: false,
            player_thruster_idx: None,
            player_body_id: None,
            texture_names: vec![
                "ship".to_string(),
                "bullet".to_string(),
                "alien".to_string(),
                "ship_thruster_spritesheet".to_string(),
                "alien_movement".to_string(),
            ],
            game_state: TestGameState::Loading,
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

    fn update_background_color(&mut self, new_color: sg::Color) {
        self.current_background = new_color;
        self.new_background = true;
    }

    fn reset_game(&mut self, camera: &mut Camera2D, physics_world: &mut PhysicsWorld) {
        // Reset game to initial state
        self.frame_count = 0;
        self.current_background = sg::Color {
            r: 0.0,
            g: 0.4,
            b: 0.7,
            a: 1.0,
        };
        self.player.position = Vec2::new(-10.0, -100.0);
        self.game_state = TestGameState::Playing;

        // Clear existing asteroids from both visual and physics
        self.asteroids.clear();
        // Note: You'll also need to clear physics bodies, but you'd need a method like:
        // physics_world.clear_all_bodies_except(self.player_body_id);

        // Regenerate asteroids
        let mut rng = rand::rng();
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

            let circle = Circle::new(x, y, radius, color).with_segments(segments);
            let collider = Collider::new_circle(x, y, radius);
            let mut body = RigidBody::new_static(BodyId(i as u32), Vec2::new(x, y), collider);

            if i == 0 {
                let gravity_field = GravityField::new(200.0, 300.0, GravityFalloff::Custom(0.001));
                body.set_gravity_field(Some(gravity_field));
            }

            let body_id = physics_world.add_body(body);
            self.asteroids.insert(body_id, circle);
        }

        let target_camera_pos = self.player.position + Vec2::new(50.0, 25.0);
        camera.set_position(target_camera_pos);
    }

    fn spawn_firework_burst(
        &self,
        center: Vec2,
        count: usize,
        speed: f32,
        life: f32,
        color: Vec4,
        particle_systems: &mut HashMap<String, ParticleSystem>,
    ) {
        // Emit ~count particles quickly (single burst)
        let burst_duration = 0.05f32.max(0.001);
        let emission_rate = (count as f32 / burst_duration).max(1.0);

        let sys = ParticleSystem::new(center, emission_rate, burst_duration, life)
            .with_velocity_radial(speed * 0.7, speed * 1.2)
            .with_fixed_color(color)
            .with_drag(0.0);

        let key = format!(
            "firework_{:.2}_{:.2}_{}",
            center.x,
            center.y,
            rand::rng().random_range(0..1_000_000)
        );
        particle_systems.insert(key, sys);
    }

    fn spawn_confetti_rain(
        &self,
        area_min: Vec2,
        area_max: Vec2,
        duration: f32,
        particle_systems: &mut HashMap<String, ParticleSystem>,
    ) {
        // Steady rain from the top edge across the width
        let width = area_max.x - area_min.x;
        let top_y = area_max.y;

        // Pick a center spawn; system will randomize velocity, we randomize color via palette
        let sys = ParticleSystem::new(
            Vec2::new(area_min.x + width * 0.5, top_y),
            120.0,
            duration,
            1.2,
        )
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
    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("My Awesome Test Game")
            .with_size(1000, 800)
            .with_background(sg::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.8,
            })
            .with_samples(4)
            .with_high_dpi(false)
    }

    fn init(&mut self, config: &GameConfig, services: &mut EngineServices) {
        self.current_background = config.background_color;
        self.new_background = true;
        services.physics.set_substeps(4);

        // Load the texture once here.
        for texture_name in &self.texture_names {
            let path = format!("assets/{}.png", texture_name);
            if let Ok(_) = services.renderer.load_texture(texture_name, &path) {
                println!("Loaded texture: {}", texture_name);
            } else {
                eprintln!("Failed to load texture: {}", texture_name);
            }
        }

        // Load font texture once
        services.renderer.load_texture("font", "assets/font.png");

        // Create a text renderer: 16x16 atlas, 8x8 glyphs (adjust to your atlas)
        self.text = Some(crate::engine::TextRenderer::new("font", 16.0, 16.0, 16, 6));

        services.register_animation(SpriteAnimations::new(
            "player_thruster".to_string(),
            "ship_thruster_spritesheet".to_string(),
            Vec2::new(32.0, 32.0),
            4,
            4,
            1.0,
            LoopType::Loop,
        ));

        services.register_animation(SpriteAnimations::new(
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
            .with_color_range(
                glam::Vec4::new(0.6, 0.0, 0.0, 1.0),
                glam::Vec4::new(1.0, 1.0, 0.0, 1.0),
            )
            .with_drag(0.4);

        services
            .particles
            .insert("player_thruster".to_string(), thruster);

        // store None for now; direct key access is preferable
        self.player_thruster_idx = Some("player_thruster".to_string());

        self.asteroids.clear(); // Clear any existing
        for i in 0..20 {
            let mut rng = rand::rng();
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

            let circle = Circle::new(x, y, radius, color).with_segments(segments);
            let collider = Collider::new_circle(x, y, radius);
            let mut body = RigidBody::new_static(BodyId(i as u32), Vec2::new(x, y), collider);

            if i == 0 {
                let gravity_field = GravityField::new(200.0, 300.0, GravityFalloff::Custom(0.001));
                body.set_gravity_field(Some(gravity_field));
            }

            let body_id = services.physics.add_body(body);
            self.asteroids.insert(body_id, circle);
        }

        // Create a dynamic body for the player
        let radius = (self.player.size.x.min(self.player.size.y)) * 0.15;
        let player_collider =
            Collider::new_circle(self.player.position.x, self.player.position.y, radius);
        let player_body =
            RigidBody::new_dynamic(BodyId(999), self.player.position, player_collider, 1.0)
                .with_restitution(0.05)
                .with_friction(0.2)
                .with_drag(0.6);
        self.player_body_id = Some(services.physics.add_body(player_body));

        println!("Game initialized!");
        println!("Window size: {}x{}", sapp::width(), sapp::height());

        self.game_state = TestGameState::MainMenu;
    }

    fn update(&mut self, dt: f32, input: &InputManager, services: &mut EngineServices) {
        if self.hud_timer > 0.0 {
            self.hud_timer -= dt;
            if self.hud_timer <= 0.0 {
                self.hud_msg = None;
            }
        }

        // Handle game state transitions
        match self.game_state {
            TestGameState::MainMenu => {
                services.update_particles(dt);

                if input.is_key_pressed(sapp::Keycode::Enter) {
                    self.game_state = TestGameState::Playing;
                    println!("Starting game!");
                }
                if input.is_key_pressed(sapp::Keycode::Escape) {
                    sapp::request_quit(); // Direct quit instead of SystemState
                }
            }
            TestGameState::Playing => {
                // Update backend services;
                services.update_physics(dt);
                services.update_particles(dt);

                // Update animations
                let mut sprites = vec![&mut self.player];
                services.update_animations(dt, &mut sprites);

                if input.is_key_pressed(sapp::Keycode::P) {
                    self.game_state = TestGameState::Paused;
                }
                if input.is_key_pressed(sapp::Keycode::M) {
                    self.game_state = TestGameState::MainMenu;
                }
                // Continue with existing game logic below

                let mut thrust_force = Vec2::ZERO;
                const THRUST_STRENGTH: f32 = 500.0; // Adjust this value to control acceleration

                if input.is_key_down(sapp::Keycode::W) {
                    thrust_force.y += THRUST_STRENGTH;
                }
                if input.is_key_down(sapp::Keycode::S) {
                    thrust_force.y -= THRUST_STRENGTH;
                }
                if input.is_key_down(sapp::Keycode::A) {
                    thrust_force.x -= THRUST_STRENGTH;
                }
                if input.is_key_down(sapp::Keycode::D) {
                    thrust_force.x += THRUST_STRENGTH;
                }

                if let Some(body_id) = self.player_body_id {
                    if let Some(body) = services.physics.get_body_mut(body_id) {
                        if thrust_force.length() > 0.0 {
                            // Apply thrust force
                            body.apply_force(thrust_force);

                            // Calculate rotation based on thrust direction
                            self.player.rotation =
                                thrust_force.y.atan2(thrust_force.x) - std::f32::consts::PI / 2.0;

                            // Store body position before releasing the borrow
                            let body_position = body.position;

                            // Sync sprite position with physics body
                            self.player.position = body_position;

                            // Keep player within world bounds by clamping physics body position
                            let half = self.player.size * 0.5;
                            let clamped_pos = Vec2::new(
                                body_position
                                    .x
                                    .clamp(self.world_min.x + half.x, self.world_max.x - half.x),
                                body_position
                                    .y
                                    .clamp(self.world_min.y + half.y, self.world_max.y - half.y),
                            );

                            if clamped_pos != body_position {
                                body.set_position(clamped_pos);
                                self.player.position = clamped_pos;
                            }
                        } else {
                            // Store body position before releasing the borrow
                            let body_position = body.position;
                            self.player.position = body_position;

                            // Keep player within world bounds
                            let half = self.player.size * 0.5;
                            let clamped_pos = Vec2::new(
                                body_position
                                    .x
                                    .clamp(self.world_min.x + half.x, self.world_max.x - half.x),
                                body_position
                                    .y
                                    .clamp(self.world_min.y + half.y, self.world_max.y - half.y),
                            );

                            if clamped_pos != body_position {
                                body.set_position(clamped_pos);
                                self.player.position = clamped_pos;
                            }
                        }
                    }

                    // Handle animation and particles AFTER releasing the physics borrow
                    if thrust_force.length() > 0.0 {
                        // Handle animation
                        if !self.player_animation {
                            self.player
                                .change_texture("ship_thruster_spritesheet".to_string());
                            services.play_animation(&mut self.player, "player_thruster");
                            self.player_animation = true;
                        }

                        // Update thruster particles
                        if let Some(key) = &self.player_thruster_idx {
                            if let Some(sys) = services.particles.get_mut(key) {
                                let backward = thrust_force.normalize();
                                let offset = Vec2::new(-backward.x, -backward.y)
                                    * (self.player.size.y * 0.4);
                                sys.set_spawn_position(self.player.position + offset);
                                sys.set_velocity_direction(
                                    Vec2::new(-backward.x, -backward.y),
                                    40.0,
                                    90.0,
                                    0.5,
                                );
                                sys.set_emission_rate(120.0);
                            }
                        }
                    } else {
                        // No thrust - turn off animation and thruster
                        self.player.change_texture("ship".to_string());
                        self.player.size = Vec2::new(32.0, 32.0);
                        self.player.uv = Vec4::new(0.0, 0.0, 1.0, 1.0);
                        self.player_animation = false;
                        services.stop_animation(&mut self.player);

                        if let Some(key) = &self.player_thruster_idx {
                            if let Some(sys) = services.particles.get_mut(key) {
                                sys.set_emission_rate(0.0);
                            }
                        }
                    }
                }

                // Collect collision information first
                let mut player_collisions = Vec::new();
                for event in services.physics.get_collision_events() {
                    if let Some(player_body_id) = self.player_body_id {
                        if event.body1_id == player_body_id {
                            player_collisions.push((event.body2_id, event.contact_point));
                        } else if event.body2_id == player_body_id {
                            player_collisions.push((event.body1_id, event.contact_point));
                        }
                    }
                }

                // Now process collisions without holding any borrows
                for (other_id, contact_point) in player_collisions {
                    // Mark asteroid for deletion
                    if let Some(hit_body) = services.physics.get_body_mut(other_id) {
                        hit_body.mark_for_deletion();
                    }

                    // Remove visual asteroid and create explosion
                    if let Some(circle) = self.asteroids.remove(&other_id) {
                        let color = circle.color;
                        services.camera.add_shake(5.0, 0.2);
                        let explosion_system = ParticleSystem::new(contact_point, 50.0, 0.2, 1.5)
                            .with_fixed_color(color);
                        let key = format!("explosion_{}", rand::rng().random_range(0..1_000_000));
                        services.particles.insert(key, explosion_system);
                    }

                    self.hud_msg = Some(format!(
                        "Asteroid hit! {} asteroids remaining",
                        self.asteroids.len()
                    ));
                    self.hud_timer = 1.5;
                }

                services.update_animations(dt, &mut vec![&mut self.player]);

                // Camera follows the box with some offset
                let target_camera_pos = self.player.position + Vec2::new(50.0, 25.0);
                services.camera.set_position(target_camera_pos);
                services
                    .camera
                    .clamp_to_bounds(self.world_min, self.world_max);

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
                if input.is_key_pressed(sapp::Keycode::Escape) {}
            }
            TestGameState::Dead => {
                if input.is_key_pressed(sapp::Keycode::Enter) {
                    self.reset_game(services.camera, services.physics);
                }
                if input.is_key_pressed(sapp::Keycode::Escape) {
                    self.game_state = TestGameState::MainMenu;
                }
            }
            TestGameState::Completed => {
                // Center camera for the celebration
                services.camera.set_position(Vec2::ZERO);

                // Start sequence once
                if !self.completed_fx_started {
                    self.completed_fx_started = true;
                    self.completed_fx_timer = 0.0;
                    self.completed_fx_next_burst = 0.0;

                    // Initial shake and triple bursts at center
                    services.camera.add_shake(6.0, 0.4);
                    let colors = [
                        Vec4::new(1.0, 0.3, 0.3, 1.0),
                        Vec4::new(1.0, 0.8, 0.2, 1.0),
                        Vec4::new(0.3, 0.8, 1.0, 1.0),
                    ];
                    for (i, c) in colors.iter().enumerate() {
                        let speed = 180.0 + (i as f32) * 60.0;
                        let life = 1.0 + (i as f32) * 0.25;
                        self.spawn_firework_burst(
                            Vec2::ZERO,
                            28,
                            speed,
                            life,
                            *c,
                            services.particles,
                        );
                    }

                    // Short confetti rain across a visible area
                    self.spawn_confetti_rain(
                        Vec2::new(-300.0, -200.0),
                        Vec2::new(300.0, 200.0),
                        0.8,
                        services.particles,
                    );
                }

                // Run timed bursts for ~2.5s
                self.completed_fx_timer += dt;
                if self.completed_fx_timer >= self.completed_fx_next_burst
                    && self.completed_fx_timer < 2.5
                {
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
                    self.spawn_firework_burst(pos, 20, 160.0, 1.0, color, services.particles);
                    self.spawn_firework_burst(
                        -pos * 0.5,
                        16,
                        140.0,
                        0.8,
                        color,
                        services.particles,
                    );

                    // subtle micro-shake
                    services.camera.add_shake(2.0, 0.1);
                }

                if input.is_key_pressed(sapp::Keycode::M) {
                    self.game_state = TestGameState::MainMenu;
                    self.completed_fx_started = false;
                }
                if input.is_key_pressed(sapp::Keycode::Enter) {
                    self.reset_game(services.camera, services.physics);
                    self.completed_fx_started = false;
                }
            }
            TestGameState::Loading => {
                // Handle loading/transition logic
            }
        }
    }

    fn render(&mut self, services: &mut EngineServices) {
        services.begin_frame();

        match self.game_state {
            TestGameState::MainMenu => {
                // Render main menu
                let menu_bg =
                    Quad::new(-400.0, -300.0, 800.0, 600.0, Vec4::new(0.1, 0.1, 0.3, 0.9));
                services.renderer.draw_quad(&menu_bg);

                let title_box =
                    Quad::new(-150.0, 100.0, 300.0, 80.0, Vec4::new(0.8, 0.8, 0.0, 1.0));
                services.renderer.draw_quad(&title_box);

                let start_button =
                    Quad::new(-100.0, 0.0, 200.0, 50.0, Vec4::new(0.0, 0.8, 0.0, 1.0));
                services.renderer.draw_quad(&start_button);

                if let Some(text) = &self.text {
                    let mut t = text.clone();
                    t.set_scale(3.0);
                    t.set_color(Vec4::new(1.0, 1.0, 0.2, 1.0));
                    // Test with simple characters first
                    t.draw_text_world(services.renderer, glam::Vec2::new(-140.0, 120.0), "ABC");
                    t.set_scale(2.0);
                    t.set_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
                    t.draw_text_world(services.renderer, glam::Vec2::new(-90.0, 10.0), "123");
                }
            }
            TestGameState::Playing => {
                //draw window bounds
                let world_size = self.world_max - self.world_min;
                let border = Quad::new(
                    self.world_min.x,
                    self.world_min.y,
                    world_size.x,
                    world_size.y,
                    Vec4::new(0.1, 0.1, 0.3, 0.9),
                )
                .with_outline();
                services.renderer.draw_quad(&border);

                for astroid in self.asteroids.values() {
                    services.renderer.draw_circle(astroid);
                }

                services.renderer.draw_sprite(&self.player);

                if let (Some(text), Some(msg)) = (&self.text, &self.hud_msg) {
                    if self.hud_timer > 0.0 {
                        let mut t = text.clone();
                        t.set_scale(1.0);
                        t.set_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
                        t.draw_top_right(
                            services.renderer,
                            services.camera,
                            Vec2::new(20.0, 20.0),
                            msg,
                        );
                    }
                }

                services.render_particles();
                services.render_physics_debug();
            }
            TestGameState::Paused => {
                // Render game in background + pause overlay
                for astroid in self.asteroids.values() {
                    services.renderer.draw_circle(astroid);
                }
                services.renderer.draw_sprite(&self.player);

                // Pause overlay
                let pause_overlay = Quad::new(
                    services.camera.get_position().x - 400.0,
                    services.camera.get_position().y - 300.0,
                    800.0,
                    600.0,
                    Vec4::new(0.0, 0.0, 0.0, 0.5),
                );
                services.renderer.draw_quad(&pause_overlay);

                let pause_text = Quad::new(
                    services.camera.get_position().x - 100.0,
                    services.camera.get_position().y,
                    200.0,
                    50.0,
                    Vec4::new(1.0, 1.0, 1.0, 1.0),
                );
                services.renderer.draw_quad(&pause_text);
            }
            TestGameState::Completed => {
                // Pause overlay
                let completed_overlay = Quad::new(
                    services.camera.get_position().x - 400.0,
                    services.camera.get_position().y - 300.0,
                    800.0,
                    600.0,
                    Vec4::new(0.0, 0.0, 0.0, 0.5),
                );
                services.renderer.draw_quad(&completed_overlay);

                let completed_text = Quad::new(
                    services.camera.get_position().x - 100.0,
                    services.camera.get_position().y,
                    200.0,
                    50.0,
                    Vec4::new(1.0, 1.0, 1.0, 1.0),
                );
                services.renderer.draw_quad(&completed_text);
            }
            _ => {
                // Handle other states as needed
            }
        }
    }

    // handle events that are not movement based
    fn handle_event(&mut self, event: &sapp::Event) {
        match event._type {
            sapp::EventType::KeyDown => match event.key_code {
                sapp::Keycode::F1 => {
                    println!("F1, toggle debug information!");
                }
                _ => {}
            },
            sapp::EventType::Unfocused => {
                println!("Game suspended!");
                self.game_state = TestGameState::Paused;
            }
            sapp::EventType::Focused => {
                println!("Game active!");
            }
            _ => {}
        }
    }

    fn request_background_color_change(&self) -> Option<sg::Color> {
        if self.new_background {
            return Some(self.current_background);
        }
        None
    }

    fn engine_render_loading(
        &mut self,
        renderer: &mut Renderer,
        progress: f32,
        _camera: &mut Camera2D,
    ) {
        // Dark space background
        let bg = Quad::new(-400.0, -300.0, 800.0, 600.0, Vec4::new(0.0, 0.0, 0.1, 1.0));
        renderer.draw_quad(&bg);

        // Loading bar background
        let bar_bg = Quad::new(-200.0, -20.0, 400.0, 40.0, Vec4::new(0.2, 0.2, 0.3, 1.0));
        renderer.draw_quad(&bar_bg);

        // Loading bar fill (progress-based width)
        let bar_fill = Quad::new(
            -200.0,
            -20.0,
            400.0 * progress,
            40.0,
            Vec4::new(0.0, 0.6, 1.0, 1.0),
        );
        renderer.draw_quad(&bar_fill);

        // Title
        if let Some(text) = &self.text {
            let mut t = text.clone();
            t.set_scale(2.5);
            t.set_color(Vec4::new(1.0, 0.8, 0.2, 1.0));
            t.draw_text_world(renderer, Vec2::new(-120.0, 80.0), "ASTEROIDS");

            // Loading percentage
            t.set_scale(1.5);
            t.set_color(Vec4::new(0.8, 0.8, 0.8, 1.0));
            let percent = (progress * 100.0) as i32;
            t.draw_text_world(renderer, Vec2::new(-40.0, -60.0), &format!("{}%", percent));
        }
    }
}
