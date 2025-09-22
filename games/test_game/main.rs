use engine::App;
use rusclog::engine::{self, toggle_collision_debug, toggle_debug_text};

use crate::test_game::TestGame;

pub mod test_game;

fn main() {
    let game = TestGame::new();
    let app = App::new(game);

    toggle_debug_text();
    toggle_collision_debug();

    app.run();
}
