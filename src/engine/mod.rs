pub mod app;
pub mod graphics;
pub mod input;
pub mod camera;
pub mod animation;
pub mod collision;
pub mod texture;
pub mod particle;

use sokol::gfx as sg;
pub use app::*;
pub use input::*;
pub use camera::*;
pub use graphics::*;
pub use animation::*;
pub use collision::*;
pub use texture::*;
pub use particle::*;

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
    fn default() -> Self {
        Self { 
            window_title: "My new Game".to_string(), 
            window_width: 800, 
            window_height: 600, 
            background_color: sg::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
            sample_count: 1, 
            high_dpi: false 
        }
    }
}

impl GameConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.window_title = title.to_string();
        self
    }

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

// Trait that games must implement
pub trait Game {
    fn config() -> GameConfig where Self: Sized;

    fn init(&mut self, config: &GameConfig, renderer: &mut Renderer, animation_manager: &mut AnimationManager);
    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D, animation_manager: &mut AnimationManager, particle_systems: &mut Vec<ParticleSystem>);

    fn render(&mut self, _renderer: &mut Renderer, camera: &mut Camera2D, particle_systems: &mut Vec<ParticleSystem>);

    fn handle_event(&mut self, event: &sokol::app::Event);

    fn get_background_color(&self) -> Option<sg::Color> {
        None  
    }
}