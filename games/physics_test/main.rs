use engine::App;
use rusclog::engine::{self, toggle_collision_debug, toggle_debug_text};

use crate::physic_game::PhysicsGame;

pub mod physics_game;

fn main() {
    let game = PhysicsGame::new();
    let app = App::new(game);

    toggle_debug_text();
    toggle_collision_debug();

    app.run();
}
