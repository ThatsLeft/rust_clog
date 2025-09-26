use crate::engine::{
    Camera2D, Circle, Collider, Game, GameConfig, InputManager,
    LoopType::{self},
    ParticleSystem, Quad, Sprite, SpriteAnimations,
};
use glam::{Vec2, Vec4};
use rand::Rng;
use rusclog::{
    debug_print,
    engine::{
        gravity::{GravityFalloff, GravityField},
        physics_world::PhysicsWorld,
        rigid_body::{BodyId, RigidBody},
        EngineServices, TextRenderer,
    },
};
use sokol::{
    app::{self as sapp},
    gfx as sg,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsGameState {
    InitialLoading,
    Playing,
}

pub struct PhysicsGame {
    game_state: PhysicsGameState,
    current_background: sg::Color,
    new_background: bool,
    balls: HashMap<BodyId, Circle>,
    platforms: HashMap<BodyId, Quad>,
    world_min: Vec2,
    world_max: Vec2,
    text: Option<TextRenderer>,
    hud_msg: Option<String>,
    hud_timer: f32,
    loading_timer: f32,
    loading_duration: f32,
}

impl PhysicsGame {
    pub fn new() -> Self {
        Self {
            game_state: PhysicsGameState::InitialLoading,
            current_background: sg::Color {
                r: 0.0,
                g: 0.4,
                b: 0.7,
                a: 1.0,
            },
            new_background: false,
            balls: HashMap::new(),
            platforms: HashMap::new(),
            world_min: Vec2::new(-1000.0, -750.0),
            world_max: Vec2::new(1000.0, 750.0),
            text: None,
            hud_msg: None,
            hud_timer: 0.0,
            loading_timer: 0.0,
            loading_duration: 2.0,
        }
    }

    fn add_ball(&mut self, position: Vec2, services: &mut EngineServices) {
        let ball_id = self.balls.len() as u32 + 1.0 as u32;
        let mut rng = rand::rng();

        let colors = [
            Vec4::new(1.0, 0.5, 0.0, 1.0), // Orange
            Vec4::new(0.5, 0.0, 1.0, 1.0), // Purple
            Vec4::new(0.0, 1.0, 1.0, 1.0), // Cyan
            Vec4::new(1.0, 1.0, 0.0, 1.0), // Yellow
            Vec4::new(1.0, 0.0, 0.5, 1.0), // Pink
            Vec4::new(0.0, 1.0, 0.5, 1.0), // Green
            Vec4::new(0.5, 1.0, 0.0, 1.0), // Lime
            Vec4::new(1.0, 0.0, 0.0, 1.0), // Red
        ];
        let color = colors[rng.random_range(0..colors.len())];

        let radius = rng.random_range(5.0..=75.0);

        let mass = 1200.0; //rng.random_range(500.0..=1200.0);

        let circle = Circle::new(position.x, position.y, radius, color)
            .with_line(0.0)
            .with_line_color(Vec4::new(1.0, 1.0, 1.0, 1.0));
        let collider = Collider::new_circle(position.x, position.y, radius);
        let body = RigidBody::new_dynamic(BodyId(ball_id), position, collider, mass)
            .with_restitution(0.8)
            .with_friction(0.2);

        let body_id = services.physics.add_body(body);
        self.balls.insert(body_id, circle);
    }

    fn add_world_platform(&mut self, services: &mut EngineServices) {
        let platform_pos = Vec2::new(0.0, -170.0);
        let platform_size = Vec2::new(400.0, 50.0);

        // Both quad and collider should use the same center position
        let platform_quad = Quad::new(
            platform_pos.x,
            platform_pos.y,
            platform_size.x,
            platform_size.y,
            Vec4::new(0.8, 0.3, 0.1, 1.0),
        );

        // Create collider at center position
        let platform_collider = Collider::new_rect(
            platform_pos.x,
            platform_pos.y,
            platform_size.x,
            platform_size.y,
        );

        let mut platform_body =
            RigidBody::new_static(BodyId(0), platform_pos, platform_collider).with_restitution(0.2);
        // Ensure the collider position matches the body position
        platform_body.collider.position = platform_pos;

        let platform_id = services.physics.add_body(platform_body);
        self.platforms.insert(platform_id, platform_quad);
    }

    fn render_startup_loading(&mut self, services: &mut EngineServices) {
        // Dark space background
        let bg = Quad::new(0.0, 0.0, 800.0, 600.0, Vec4::new(0.0, 0.0, 0.1, 1.0));
        services.renderer.draw_quad(&bg);

        // Calculate progress
        let progress = (self.loading_timer / self.loading_duration).min(1.0);

        // Loading bar background
        let bar_bg = Quad::new(0.0, -20.0, 400.0, 40.0, Vec4::new(0.2, 0.2, 0.3, 1.0));
        services.renderer.draw_quad(&bar_bg);

        // Loading bar fill
        let bar_fill = Quad::new(
            -200.0 + (400.0 * progress * 0.5), // Center the fill properly
            -20.0,
            400.0 * progress,
            40.0,
            Vec4::new(0.0, 0.6, 1.0, 1.0),
        );
        services.renderer.draw_quad(&bar_fill);

        // Title and text
        if let Some(text) = &self.text {
            let mut t = text.clone();
            t.set_scale(2.5);
            t.set_color(Vec4::new(1.0, 0.8, 0.2, 1.0));

            let title_size = t.measure_single_line_px("PHYSICS");
            // Calculate centered position
            let center_x = 0.0; // Your desired center point (world coordinates)
            let center_y = 80.0;
            let centered_pos = Vec2::new(
                center_x - title_size.x * 0.5, // Move left by half the width
                center_y,                      // Keep the same Y
            );

            // Draw at the calculated position
            t.draw_text_world(services.renderer, centered_pos, "PHYSICS");

            t.set_scale(1.5);
            t.set_color(Vec4::new(0.8, 0.8, 0.8, 1.0));
            let percent = (progress * 100.0) as i32;
            t.draw_text_world(
                services.renderer,
                Vec2::new(-40.0, -60.0),
                &format!("{}%", percent),
            );

            // Loading status text
            t.set_scale(1.0);
            t.set_color(Vec4::new(0.6, 0.6, 0.6, 1.0));
            t.draw_text_world(
                services.renderer,
                Vec2::new(-120.0, -100.0),
                "Initializing...",
            );
        }
    }
}

impl Game for PhysicsGame {
    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("Physics test")
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
        services.physics.set_global_gravity(Vec2::new(0.0, -685.0));
        services.physics.set_substeps(8);

        // Load font texture once
        _ = services
            .renderer
            .load_texture("font", "games/test_game/assets/font.png");

        // Create a text renderer: 16x16 atlas, 8x8 glyphs (adjust to your atlas)
        self.text = Some(crate::engine::TextRenderer::new("font", 16.0, 16.0, 16, 6));

        self.add_world_platform(services);

        debug_print!("Game initialized!");
        debug_print!("Window size: {}x{}", sapp::width(), sapp::height());
    }

    fn update(&mut self, dt: f32, input: &InputManager, services: &mut EngineServices) {
        if self.hud_timer > 0.0 {
            self.hud_timer -= dt;
            if self.hud_timer <= 0.0 {
                self.hud_msg = None;
            }
        }

        match self.game_state {
            PhysicsGameState::InitialLoading => {
                self.loading_timer += dt;
                if self.loading_timer >= self.loading_duration {
                    self.game_state = PhysicsGameState::Playing;
                }
            }
            PhysicsGameState::Playing => {
                services.update_physics(dt);
                services.update_particles(dt);

                for (body_id, circle) in &mut self.balls {
                    if let Some(body) = services.physics.get_body(*body_id) {
                        circle.center = body.position;
                        circle.line_angle = body.rotation;
                    }
                }

                if input.is_mouse_button_pressed(sapp::Mousebutton::Left) {
                    let mouse_pos = input.mouse_position();
                    // Convert screen coordinates to world coordinates
                    let world_pos = services.camera.screen_to_world(mouse_pos);
                    self.add_ball(world_pos, services);

                    debug_print!(
                        "Mouse clicked at screen position: ({:.2}, {:.2})",
                        mouse_pos.x,
                        mouse_pos.y
                    );
                }

                let collision_events = services.physics.get_collision_events();
                debug_print!("Collisions detected {}", collision_events.len());

                self.hud_msg = Some(format!("Balls: {}", self.balls.len()));
                self.hud_timer = 0.5;

                services.remove_marked_bodies();
            }
        }
    }

    fn render(&mut self, services: &mut EngineServices) {
        services.begin_frame();

        match self.game_state {
            PhysicsGameState::InitialLoading => {
                self.render_startup_loading(services);
            }
            PhysicsGameState::Playing => {
                let world_size = self.world_max - self.world_min;
                let border = Quad::new(
                    (self.world_min.x + self.world_max.x) * 0.5, // Center X
                    (self.world_min.y + self.world_max.y) * 0.5, // Center Y
                    world_size.x,
                    world_size.y,
                    Vec4::new(0.1, 0.1, 0.3, 0.9),
                )
                .with_outline();
                services.renderer.draw_quad(&border);

                for ball in self.balls.values() {
                    services.renderer.draw_circle(ball);
                }

                for platform in self.platforms.values() {
                    services.renderer.draw_quad(platform);
                }

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
        }
    }

    fn handle_event(&mut self, event: &sokol::app::Event) {
        match event._type {
            sapp::EventType::KeyDown => match event.key_code {
                sapp::Keycode::F1 => {
                    println!("F1, toggle debug information!");
                }
                _ => {}
            },
            sapp::EventType::Unfocused => {
                println!("Game suspended!");
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
}
