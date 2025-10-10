use glam::Vec2;
use rusclog::engine::{rigid_body::BodyId, EngineServices};

pub struct Player {
    pub body_id: Option<BodyId>,
    pub speed: f32,
    pub size: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            body_id: None,
            speed: 200.0,
            size: 16.0,
        }
    }

    pub fn get_position(&self, services: &EngineServices) -> Option<Vec2> {
        self.body_id
            .and_then(|id| services.physics.get_body(id).map(|body| body.position))
    }

    pub fn apply_movement(&self, services: &mut rusclog::engine::EngineServices, direction: Vec2) {
        if let Some(body_id) = self.body_id {
            if let Some(body) = services.physics.get_body_mut(body_id) {
                let velocity = if direction.length_squared() > 0.0 {
                    direction.normalize() * self.speed
                } else {
                    Vec2::ZERO
                };
                body.velocity = velocity;
            }
        }
    }
}
