use glam::Vec2;
use rusclog::engine::{EngineServices, InputManager};
use sokol::app as sapp;

use crate::player::player::Player;

pub struct PlayerController;

impl PlayerController {
    pub fn handle_input(player: &Player, input: &InputManager, services: &mut EngineServices) {
        let mut move_dir = Vec2::ZERO;

        if input.is_key_down(sapp::Keycode::W) || input.is_key_down(sapp::Keycode::Up) {
            move_dir.y += 1.0;
        }
        if input.is_key_down(sapp::Keycode::S) || input.is_key_down(sapp::Keycode::Down) {
            move_dir.y -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::A) || input.is_key_down(sapp::Keycode::Left) {
            move_dir.x -= 1.0;
        }
        if input.is_key_down(sapp::Keycode::D) || input.is_key_down(sapp::Keycode::Right) {
            move_dir.x += 1.0;
        }

        player.apply_movement(services, move_dir);
    }

    pub fn update_camera(player: &Player, services: &mut EngineServices) {
        if let Some(position) = player.get_position(services) {
            services.camera.set_position(position);
        }
    }
}
