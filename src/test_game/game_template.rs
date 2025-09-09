use glam::Vec4;
use sokol::gfx as sg;

use crate::engine::{Game, GameConfig, GameState, ParticleSystem, Quad, SystemState};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateGameState {
    MainMenu,
    Playing,
    Paused,
    Loading
}

impl GameState for TemplateGameState {
    fn default_active() -> Self {
        Self::MainMenu
    }
}

pub struct TemplateGame {
    game_state: TemplateGameState,
    requested_system_state: Option<SystemState>,
}

/// This is the implementation space for your game. All functions for your game goes in this
impl TemplateGame {
    pub fn new() -> Self {
        Self {
            game_state: TemplateGameState::Loading,
            requested_system_state: None,
        }
    }
}

/// These are the game functions called by the game.
/// You can implement game features directly in this, or in the Templateimplementationt
impl Game for TemplateGame {
    type State = TemplateGameState;

    /// Configure your game window.
    fn config() -> GameConfig {
        GameConfig::new()
            .with_title("My Template Game")
            .with_size(1000, 800)
            .with_background(sg::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 })
            .with_samples(4)
            .with_high_dpi(false)
    }

    /// The inital loading screen for when the game starts.
    fn engine_render_loading(&mut self, renderer: &mut crate::engine::Renderer, progress: f32) {
        let loading_box = Quad::new(-200.0, -50.0, 400.0 * progress, 20.0, Vec4::new(0.0, 1.0, 0.0, 1.0));
        renderer.draw_quad(&loading_box);
    }
    
    /// Init your game.
    /// Load textures, and or other assets.
    fn init(&mut self, config: &GameConfig, renderer: &mut crate::engine::Renderer, animation_manager: &mut crate::engine::AnimationManager, particle_systems: &mut HashMap<String, ParticleSystem>) {
        // ...

        self.game_state = TemplateGameState::MainMenu;
    }
    
    /// Update loop for the game.
    /// All game logic, physic and AI happens here.
    fn update(&mut self, dt: f32, input: &crate::engine::InputManager, camera: &mut crate::engine::Camera2D, animation_manager: &mut crate::engine::AnimationManager, particle_systems: &mut HashMap<String, crate::engine::ParticleSystem>) {
        match self.game_state {
            TemplateGameState::MainMenu => todo!(),
            TemplateGameState::Playing => todo!(),
            TemplateGameState::Paused => todo!(),
            TemplateGameState::Loading => todo!(),
        }
    }
    
    /// Render the window.
    fn render(&mut self, renderer: &mut crate::engine::Renderer, camera: &mut crate::engine::Camera2D, particle_systems: &mut HashMap<String, crate::engine::ParticleSystem>) {
        renderer.begin_frame();

        match self.game_state {
            TemplateGameState::MainMenu => todo!(),
            TemplateGameState::Playing => todo!(),
            TemplateGameState::Paused => todo!(),
            TemplateGameState::Loading => todo!(),
        }
    }
    
    /// Handle events fromt he enginge that are not gameplay related
    /// Deal with resizing the window, focus unfocus etc...
    fn handle_event(&mut self, event: &sokol::app::Event) {
        todo!()
    } 

    /// Change the game engins states
    fn request_system_state(&mut self) -> Option<SystemState> {
        let state = self.requested_system_state;
        self.requested_system_state = None; // Clear after returning
        state
    }

    /// Request for the Background to be changed.
    /// Optional implementation if your game need to change the backgroun color
    fn request_background_color_change(&self) -> Option<sg::Color> {
        None
    }
}
