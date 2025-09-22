use engine::App;
use rusclog::engine;

use crate::test_game::TestGame;

pub mod test_game;

fn main() {
    let game = TestGame::new();
    let app = App::new(game);

    app.run();
}
