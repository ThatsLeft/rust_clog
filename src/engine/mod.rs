pub mod app;
pub mod graphics;
pub mod input;
pub mod camera;
pub mod animation;
pub mod collision;
pub mod texture;
pub mod particle;

use glam::Vec4;
use sokol::gfx as sg;
pub use app::*;
pub use input::*;
pub use camera::*;
pub use graphics::*;
pub use animation::*;
pub use collision::*;
pub use texture::*;
pub use particle::*;

/// Game engine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemState {
    Starting,
    GameActive,
    Background,
    Shutdown,
}

pub trait GameState: Clone + Copy + PartialEq + Eq + std::fmt::Debug {
    // Games must provide a default "active" state
    fn default_active() -> Self;
}

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
            background_color: sg::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
            sample_count: 1, 
            high_dpi: false 
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

// Trait that games must implement
pub trait Game {
    type State: GameState;
    fn config() -> GameConfig where Self: Sized;
    
    fn engine_render_loading(&mut self, renderer: &mut Renderer, progress: f32);
    

    fn init(&mut self, config: &GameConfig, renderer: &mut Renderer, animation_manager: &mut AnimationManager);
    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D, animation_manager: &mut AnimationManager, particle_systems: &mut Vec<ParticleSystem>);
    
    fn render(&mut self, renderer: &mut Renderer, camera: &mut Camera2D, particle_systems: &mut Vec<ParticleSystem>);
    
    fn handle_event(&mut self, event: &sokol::app::Event);
    
    fn request_system_state(&mut self) -> Option<SystemState>;
    
    fn request_background_color_change(&self) -> Option<sg::Color> {
        None  
    }
    
}