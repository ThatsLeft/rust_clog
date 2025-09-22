pub mod animation;
pub mod app;
pub mod camera;
pub mod collision;
pub mod graphics;
pub mod input;
pub mod particle;
pub mod physics;
pub mod text;
pub mod texture;

pub use animation::*;
pub use app::*;
pub use camera::*;
pub use collision::*;
use glam::Vec4;
pub use graphics::*;
pub use input::*;
pub use particle::*;
pub use physics::*;
use sokol::gfx as sg;
use std::collections::HashMap;
pub use text::*;
pub use texture::*;

use crate::engine::physics_world::PhysicsWorld;

/// Game window configuration
/// Implemented with builder
#[derive(Clone)]
pub struct GameConfig {
    pub window_title: String,
    pub window_width: i32,
    pub window_height: i32,
    pub background_color: sg::Color,
    pub sample_count: i32,
    pub high_dpi: bool,
}

impl Default for GameConfig {
    /// GameConfig default to 800 x 600, called "My new Game"
    fn default() -> Self {
        Self {
            window_title: "My new Game".to_string(),
            window_width: 800,
            window_height: 600,
            background_color: sg::Color {
                r: 0.6,
                g: 0.6,
                b: 0.6,
                a: 1.0,
            },
            sample_count: 1,
            high_dpi: false,
        }
    }
}

impl GameConfig {
    /// Initial default implementation
    pub fn new() -> Self {
        Self::default()
    }

    /// Set your game window title
    pub fn with_title(mut self, title: &str) -> Self {
        self.window_title = title.to_string();
        self
    }

    ///
    pub fn with_size(mut self, width: i32, height: i32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    pub fn with_background(mut self, color: sg::Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn with_samples(mut self, samples: i32) -> Self {
        self.sample_count = samples;
        self
    }

    pub fn with_high_dpi(mut self, high_dpi: bool) -> Self {
        self.high_dpi = high_dpi;
        self
    }
}

pub struct EngineServices<'a> {
    pub physics: &'a mut PhysicsWorld,
    pub particles: &'a mut HashMap<String, ParticleSystem>,
    pub animation: &'a mut AnimationManager,
    pub camera: &'a mut Camera2D,
    pub renderer: &'a mut Renderer,
}

impl EngineServices<'_> {
    pub fn update_physics(&mut self, dt: f32) {
        self.physics.step(dt);
        let _removed_bodies = self.physics.remove_marked_bodies();
        self.physics.clear_collision_events();
    }

    pub fn update_particles(&mut self, dt: f32) {
        for system in self.particles.values_mut() {
            system.update(dt);
        }

        // Remove finished, duration-based systems
        let finished_keys: Vec<String> = self
            .particles
            .iter()
            .filter_map(|(k, s)| {
                if s.is_finished() {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();
        for key in finished_keys {
            self.particles.remove(&key);
        }
    }

    pub fn update_animations(&mut self, dt: f32, sprites: &mut [&mut Sprite]) {
        for sprite in sprites {
            self.animation.update_sprite_animation(sprite, dt);
        }
    }

    pub fn play_animation(&mut self, sprite: &mut Sprite, animation_name: &str) {
        self.animation.play_animation(sprite, animation_name);
    }

    pub fn stop_animation(&mut self, sprite: &mut Sprite) {
        self.animation.stop_animation(sprite);
    }

    pub fn clear_animation(&mut self, sprite: &mut Sprite) {
        self.animation.clear_animation(sprite);
    }

    pub fn register_animation(&mut self, animation: SpriteAnimations) {
        self.animation.register_animation(animation);
    }

    pub fn update_camera_shake(&mut self, dt: f32) {
        self.camera.update_shake(dt);
    }

    pub fn render_particles(&mut self) {
        for system in self.particles.values_mut() {
            for particle in system.get_particles() {
                self.renderer.draw_particle(particle);
            }
        }
    }

    pub fn render_physics_debug(&mut self) {
        for body in self.physics.bodies() {
            match body.collider.shape {
                CollisionShape::Rectangle { width, height } => {
                    let bottom_left_x = body.collider.position.x - width / 2.0;
                    let bottom_left_y = body.collider.position.y - height / 2.0;
                    let rect_outline = Quad::new(
                        bottom_left_x,
                        bottom_left_y,
                        width,
                        height,
                        Vec4::new(1.0, 0.0, 0.0, 1.0),
                    )
                    .with_outline();
                    self.renderer.draw_quad(&rect_outline);
                }
                CollisionShape::Circle { radius } => {
                    let circle_outline = Circle::new(
                        body.collider.position.x,
                        body.collider.position.y,
                        radius,
                        Vec4::new(1.0, 0.0, 0.0, 1.0),
                    )
                    .with_outline();
                    self.renderer.draw_circle(&circle_outline);
                }
            }
        }
    }

    pub fn begin_frame(&mut self) {
        self.renderer.begin_frame();
    }

    pub fn flush_and_present(&mut self) {
        self.renderer.flush(self.camera);
    }
}

// Trait that games must implement
pub trait Game {
    fn config() -> GameConfig
    where
        Self: Sized;

    fn init(&mut self, config: &GameConfig, services: &mut EngineServices);

    fn update(&mut self, dt: f32, input: &InputManager, services: &mut EngineServices);

    fn render(&mut self, services: &mut EngineServices);

    fn handle_event(&mut self, event: &sokol::app::Event);

    fn request_background_color_change(&self) -> Option<sg::Color> {
        None
    }
}
