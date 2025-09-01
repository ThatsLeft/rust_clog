pub mod app;
pub mod graphics;
pub mod input;
pub mod camera;

use sokol::gfx as sg;
pub use app::*;
pub use graphics::*;
pub use input::*;
pub use camera::*;

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
}

// Trait that games must implement
pub trait Game {
    fn config() -> GameConfig where Self: Sized;

    fn init(&mut self, config: &GameConfig);
    fn update(&mut self, dt: f32, input: &InputManager, camera: &mut Camera2D);

    fn render(&mut self);
    fn render_with_renderer(&mut self, _renderer: &mut Renderer, camera: &mut Camera2D) {  // ADD THIS
        self.render();  // Default: just call old render method
    }

    fn handle_event(&mut self, event: &sokol::app::Event);

    fn get_background_color(&self) -> Option<sg::Color> {
        None  
    }
}