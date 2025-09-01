mod engine;
mod test_game;

use engine::App;

use crate::test_game::TestGame;

fn main() {
    let game = TestGame::new();
    let app = App::new(game);
    app.run();
}