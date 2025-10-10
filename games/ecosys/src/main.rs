use rusclog::engine::{toggle_collision_debug, toggle_debug_text, App};

use crate::ecosys_game::EcosysGame;

pub mod ecosys_game;
mod player;
mod world;

fn main() {
    let game = EcosysGame::new();
    let app = App::new(game);

    #[cfg(debug_assertions)]
    {
        println!("Debug mode!");
        toggle_debug_text();
        toggle_collision_debug();
    }

    app.run();
}
