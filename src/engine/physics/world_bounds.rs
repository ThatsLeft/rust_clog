use glam::Vec2;

use crate::engine::rigid_body::BodyId;

#[derive(Debug, Clone)]
pub struct WorldBounds {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Debug, Clone)]
pub enum BoundsBehavior {
    /// Ignore bounds completely (infinit world)
    Ignore,
    /// Generate events when bounds are violated, but dont constrain
    Events,
    /// Clamp objects to bounds, hard walls
    Clamp { restitution: f32 },
    /// Wrap objects to oposite side
    Wrap,
    /// Mark obejct for deletion when out of bounds
    Delete { safety_margin: f32 },
    /// Custom per-body behavior
    PerBody,
}

#[derive(Debug, Clone)]
pub struct BoundsEvent {
    pub body_id: BodyId,
    pub position: Vec2,
    pub violation: BoundsViolation,
}

#[derive(Debug, Clone)]
pub enum BoundsViolation {
    Left(f32),
    Right(f32),
    Top(f32),
    Bottom(f32),
}
