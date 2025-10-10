use glam::{Vec2, Vec4};
use rand::Rng;
use rusclog::{
    debug_print,
    engine::{
        rigid_body::{BodyId, RigidBody},
        world_bounds::{BoundsBehavior, WorldBounds},
        Collider, EngineServices, Game, GameConfig, ParticleSystem, Quad, TextRenderer,
    },
};
use sokol::{
    app::{self as sapp},
    gfx as sg,
};

use crate::{
    player::{player::Player, player_controller::PlayerController},
    world::grass::Grass,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcosysGameState {
    InitialLoading,
    Playing,
}

pub struct EcosysGame {
    game_state: EcosysGameState,
    current_background: sg::Color,
    new_background: bool,
    world_min: Vec2,
    world_max: Vec2,
    text: Option<TextRenderer>,
    hud_msg: Option<String>,
    hud_timer: f32,
    loading_timer: f32,
    loading_duration: f32,

    border_particle_timer: f32,
    border_wave_offset: f32,

    player: Player,
    grass_patches: Vec<Grass>,
}

impl EcosysGame {
    pub fn new() -> Self {
        Self {
            game_state: EcosysGameState::InitialLoading,
            current_background: sg::Color {
                r: 0.0,
                g: 0.4,
                b: 0.7,
                a: 1.0,
            },
            new_background: false,
            world_min: Vec2::new(-1000.0, -750.0),
            world_max: Vec2::new(1000.0, 750.0),
            text: None,
            hud_msg: None,
            hud_timer: 0.0,
            loading_timer: 0.0,
            loading_duration: 2.0,

            border_particle_timer: 0.0,
            border_wave_offset: 0.0,

            player: Player::new(),
            grass_patches: Vec::new(),
        }
    }

    fn render_startup_loading(&mut self, services: &mut EngineServices) {
        let title = "Ecosys";

        // Dark space background
        let bg = Quad::new(0.0, 0.0, 800.0, 600.0, Vec4::new(0.0, 0.0, 0.1, 1.0));
        services.renderer.draw_quad(&bg);

        // Calculate progress
        let progress = (self.loading_timer / self.loading_duration).min(1.0);

        // Loading bar background
        let bar_bg = Quad::new(0.0, -10.0, 400.0, 40.0, Vec4::new(0.2, 0.2, 0.3, 1.0));
        services.renderer.draw_quad(&bar_bg);

        // Loading bar fill
        let bar_fill = Quad::new(
            -200.0 + (400.0 * progress * 0.5), // Center the fill properly
            -10.0,
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

            let title_size = t.measure_single_line_px(title);
            // Calculate centered position
            let center_x = 0.0; // Your desired center point (world coordinates)
            let center_y = 70.0;
            let centered_pos = Vec2::new(
                center_x - title_size.x * 0.5, // Move left by half the width
                center_y,                      // Keep the same Y
            );

            // Draw at the calculated position
            t.draw_text_world(services.renderer, centered_pos, title);

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

    fn render_world_border(&self, services: &mut EngineServices) {
        let world_size = self.world_max - self.world_min;
        let world_center = Vec2::new(
            (self.world_min.x + self.world_max.x) * 0.5,
            (self.world_min.y + self.world_max.y) * 0.5,
        );

        // Draw main world
        let border = Quad::new(
            world_center.x,
            world_center.y,
            world_size.x,
            world_size.y,
            Vec4::new(0.478, 0.282, 0.255, 1.0),
        );
        services.renderer.draw_quad(&border);

        // Draw gradient fade layers at edges - creates soft boundary
        let fade_width = 80.0;
        let num_layers = 10;

        for i in 0..num_layers {
            let progress = i as f32 / num_layers as f32;
            let alpha = 1.0 - progress; // Fade out
            let offset = progress * fade_width;

            let fade_color = Vec4::new(0.478, 0.282, 0.255, alpha);
            let layer_thickness = fade_width / num_layers as f32;

            let current_offset = offset + layer_thickness * 0.5;

            // Top edge (without corners)
            let top_fade = Quad::new(
                world_center.x,
                self.world_max.y + current_offset,
                world_size.x,
                layer_thickness,
                fade_color,
            );
            services.renderer.draw_quad(&top_fade);

            // Bottom edge (without corners)
            let bottom_fade = Quad::new(
                world_center.x,
                self.world_min.y - current_offset,
                world_size.x,
                layer_thickness,
                fade_color,
            );
            services.renderer.draw_quad(&bottom_fade);

            // Left edge (without corners)
            let left_fade = Quad::new(
                self.world_min.x - current_offset,
                world_center.y,
                layer_thickness,
                world_size.y,
                fade_color,
            );
            services.renderer.draw_quad(&left_fade);

            // Right edge (without corners)
            let right_fade = Quad::new(
                self.world_max.x + current_offset,
                world_center.y,
                layer_thickness,
                world_size.y,
                fade_color,
            );
            services.renderer.draw_quad(&right_fade);
        }

        // Draw corner gradients - fill diagonally layer by layer
        for i in 0..num_layers {
            for j in 0..num_layers {
                // Use the maximum progress for diagonal fade
                let progress_x = i as f32 / num_layers as f32;
                let progress_y = j as f32 / num_layers as f32;
                let progress = progress_x.max(progress_y); // Diagonal fade

                let alpha = 1.0 - progress;
                let offset_x = progress_x * fade_width;
                let offset_y = progress_y * fade_width;

                let fade_color = Vec4::new(0.478, 0.282, 0.255, alpha);
                let layer_thickness = fade_width / num_layers as f32;

                let current_offset_x = offset_x + layer_thickness * 0.5;
                let current_offset_y = offset_y + layer_thickness * 0.5;

                // Top-left corner
                let tl_corner = Quad::new(
                    self.world_min.x - current_offset_x,
                    self.world_max.y + current_offset_y,
                    layer_thickness,
                    layer_thickness,
                    fade_color,
                );
                services.renderer.draw_quad(&tl_corner);

                // Top-right corner
                let tr_corner = Quad::new(
                    self.world_max.x + current_offset_x,
                    self.world_max.y + current_offset_y,
                    layer_thickness,
                    layer_thickness,
                    fade_color,
                );
                services.renderer.draw_quad(&tr_corner);

                // Bottom-left corner
                let bl_corner = Quad::new(
                    self.world_min.x - current_offset_x,
                    self.world_min.y - current_offset_y,
                    layer_thickness,
                    layer_thickness,
                    fade_color,
                );
                services.renderer.draw_quad(&bl_corner);

                // Bottom-right corner
                let br_corner = Quad::new(
                    self.world_max.x + current_offset_x,
                    self.world_min.y - current_offset_y,
                    layer_thickness,
                    layer_thickness,
                    fade_color,
                );
                services.renderer.draw_quad(&br_corner);
            }
        }
    }
}

impl Game for EcosysGame {
    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("Ecosys")
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

    fn init(&mut self, config: &GameConfig, services: &mut rusclog::engine::EngineServices) {
        self.current_background = config.background_color;
        self.new_background = true;
        services.physics.set_substeps(4);
        services.physics.set_world_bounds(
            Some(WorldBounds {
                min: self.world_min,
                max: self.world_max,
            }),
            BoundsBehavior::Wrap,
        );

        // Load font texture once
        _ = services
            .renderer
            .load_texture("font", "games/test_game/assets/font.png");

        // Create a text renderer: 16x16 atlas, 8x8 glyphs (adjust to your atlas)
        self.text = Some(TextRenderer::new("font", 16.0, 16.0, 16, 6));

        // Spawn grass
        let mut rng = rand::rng();
        for _ in 0..100 {
            let x = rng.random_range(self.world_min.x + 10.0..self.world_max.x - 10.0);
            let y = rng.random_range(self.world_min.y + 10.0..self.world_max.y - 10.0);
            self.grass_patches.push(Grass::new(Vec2::new(x, y)));
        }

        // Create player
        let spawn_position = Vec2::ZERO;
        let player_collider =
            Collider::new_circle(spawn_position.x, spawn_position.y, self.player.size * 0.5);
        let player_body = RigidBody::new_kinematic(BodyId(1000), spawn_position, player_collider);
        let body_id = services.physics.add_body(player_body);
        self.player.body_id = Some(body_id);

        debug_print!("Game initialized!");
        debug_print!("Window size: {}x{}", sapp::width(), sapp::height());
    }

    fn update(
        &mut self,
        dt: f32,
        input: &rusclog::engine::InputManager,
        services: &mut rusclog::engine::EngineServices,
    ) {
        if self.hud_timer > 0.0 {
            self.hud_timer -= dt;
            if self.hud_timer <= 0.0 {
                self.hud_msg = None;
            }
        }

        match self.game_state {
            EcosysGameState::InitialLoading => {
                self.loading_timer += dt;
                if self.loading_timer >= self.loading_duration {
                    self.game_state = EcosysGameState::Playing;
                }
            }
            EcosysGameState::Playing => {
                // update systems
                services.update_physics(dt);
                services.update_particles(dt);

                // Spawn border flicker particles
                // Spawn border creep particles with organic movement
                self.border_particle_timer += dt;
                self.border_wave_offset += dt * 2.0; // Animate wave over time

                if self.border_particle_timer >= 0.005 {
                    // Very frequent
                    self.border_particle_timer = 0.0;

                    let mut rng = rand::rng();

                    // Spawn many particles per frame for thick fog
                    for _ in 0..8 {
                        // Choose random edge
                        let (x, y, edge_normal) = match rng.random_range(0..4) {
                            0 => {
                                let x = rng.random_range(self.world_min.x..self.world_max.x);
                                (x, self.world_max.y, Vec2::new(0.0, -1.0))
                            }
                            1 => {
                                let x = rng.random_range(self.world_min.x..self.world_max.x);
                                (x, self.world_min.y, Vec2::new(0.0, 1.0))
                            }
                            2 => {
                                let y = rng.random_range(self.world_min.y..self.world_max.y);
                                (self.world_min.x, y, Vec2::new(1.0, 0.0))
                            }
                            _ => {
                                let y = rng.random_range(self.world_min.y..self.world_max.y);
                                (self.world_max.x, y, Vec2::new(-1.0, 0.0))
                            }
                        };

                        // Wave motion
                        let wave_amplitude = 50.0; // Bigger wave
                        let wave_frequency = 0.003;
                        let wave =
                            (x * wave_frequency + y * wave_frequency + self.border_wave_offset)
                                .sin()
                                * wave_amplitude;

                        // Spawn particles OUTSIDE the border, moving in
                        let perpendicular = Vec2::new(-edge_normal.y, edge_normal.x);
                        let depth_offset = rng.random_range(0.0..80.0); // Spawn particles in a thick band
                        let spawn_pos = Vec2::new(x, y)
                            - edge_normal * depth_offset // Start outside the border
                            + perpendicular * wave;

                        // Direction: mostly inward with wave influence
                        let inward_strength = 0.85;
                        let wave_strength = 0.15;
                        let direction = (edge_normal * inward_strength
                            + perpendicular * wave_strength * wave.signum())
                        .normalize();

                        let border_creep = ParticleSystem::new(
                            spawn_pos, 30.0, // More particles per burst
                            0.2,  // Longer burst
                            4.0,  // Longer lifetime - particles travel further
                        )
                        .with_size_range(8.0, 20.0) // MUCH bigger particles for overlap
                        .with_color_palette(vec![
                            Vec4::new(0.0, 0.0, 0.0, 0.6), // Semi-transparent for fog
                            Vec4::new(0.05, 0.05, 0.05, 0.7),
                            Vec4::new(0.1, 0.05, 0.05, 0.5),
                            Vec4::new(0.0, 0.0, 0.0, 0.8),
                            Vec4::new(0.08, 0.04, 0.04, 0.65),
                            Vec4::new(0.02, 0.02, 0.02, 0.75),
                        ])
                        .with_velocity_direction(
                            direction,
                            8.0 + wave.abs() * 0.2, // Faster movement
                            25.0 + wave.abs() * 0.3,
                            0.8, // More spread for fog effect
                        )
                        .with_drag(0.5); // Slower drag - travel further

                        let key = format!("border_creep_{}", rng.random_range(0..1_000_000));
                        services.particles.insert(key, border_creep);
                    }
                }

                // Update grass and collect spread attempts
                let mut new_grass = Vec::new();
                for grass in &mut self.grass_patches {
                    grass.update(dt);

                    // Try to spread
                    if let Some(new_pos) = grass.try_spread() {
                        // Check if position is within bounds
                        if new_pos.x >= self.world_min.x + 10.0
                            && new_pos.x <= self.world_max.x - 10.0
                            && new_pos.y >= self.world_min.y + 10.0
                            && new_pos.y <= self.world_max.y - 10.0
                        {
                            new_grass.push(new_pos);
                        }
                    }
                }

                // Check spacing and add new grass patches
                for new_pos in new_grass {
                    // Check if not too close to existing grass
                    let too_close = self
                        .grass_patches
                        .iter()
                        .any(|g| (g.position - new_pos).length() < 12.0);

                    if !too_close {
                        self.grass_patches.push(Grass::new(new_pos));
                    }
                }

                // Remove dead grass
                self.grass_patches.retain(|g| !g.is_dead());

                // Handle player
                PlayerController::handle_input(&self.player, input, services);
                PlayerController::update_camera(&self.player, services);

                // collect collision events
                let collision_events = services.physics.get_collision_events();

                // collect bounds events
                let bounds_event = services.physics.get_bounds_events();

                // remove physics object marked for deletion
                services.remove_marked_bodies();
            }
        }
    }

    fn render(&mut self, services: &mut rusclog::engine::EngineServices) {
        services.begin_frame();

        match self.game_state {
            EcosysGameState::InitialLoading => {
                self.render_startup_loading(services);
            }
            EcosysGameState::Playing => {
                self.render_world_border(services);

                // Draw grass
                for grass in &self.grass_patches {
                    let size = 8.0 + (grass.growth * 8.0); // 8-16 pixels based on growth
                    let alpha = 0.4 + (grass.growth * 0.6); // More visible when grown
                    let color = Vec4::new(0.2, 0.6 + (grass.growth * 0.2), 0.2, alpha);

                    let grass_quad =
                        Quad::new(grass.position.x, grass.position.y, size, size, color);
                    services.renderer.draw_quad(&grass_quad);
                }

                // Draw player (get position from physics)
                if let Some(position) = self.player.get_position(services) {
                    let player_quad = Quad::new(
                        position.x,
                        position.y,
                        self.player.size,
                        self.player.size,
                        Vec4::new(0.478, 0.212, 0.482, 1.0),
                    );
                    services.renderer.draw_quad(&player_quad);
                }

                // render hud
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

                // render particles and physics
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
