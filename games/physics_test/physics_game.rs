#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsGameState {
    InitalLoading,
    Playing,
}

pub struct PhysicsGame {
    game_state: PhysicsGameState,
    current_background: sg::Color,
    balls: HashMap<BodyId, Circle>,
    platforms: HashMap<BodyId, Quad>,
    world_min: Vec2,
    world_max: Vec2,
    text: Option<TextRenderer>,
    hud_msg: Option<String>,
    hud_timer: f32,
    loading_timer: f32,
    loading_duration: f32,
}

impl PhysicsGame {
    pub fn new() -> Self {
        Self {
            game_state: PhysicsGameState::InitialLoading,
        }
    }
}
