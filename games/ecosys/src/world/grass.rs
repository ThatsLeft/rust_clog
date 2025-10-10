use glam::Vec2;
use rand::Rng;

pub struct Grass {
    pub position: Vec2,
    pub growth: f32, // 0.0 to 1.0
    pub max_growth: f32,
    pub growth_rate: f32,
    pub spread_timer: f32,
    pub spread_cooldown: f32,
}

impl Grass {
    pub fn new(position: Vec2) -> Self {
        let mut rng = rand::rng();
        Self {
            position,
            growth: rng.random_range(0.3..0.7),
            max_growth: 1.0,
            growth_rate: 0.1, // grows 0.1 per second
            spread_timer: rng.random_range(3.0..6.0),
            spread_cooldown: 5.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.growth < self.max_growth {
            self.growth = (self.growth + self.growth_rate * dt).min(self.max_growth);
        }

        self.spread_timer -= dt;
    }

    pub fn can_spread(&self) -> bool {
        self.spread_timer <= 0.0 && self.growth >= 0.8 // Only mature grass spreads
    }

    pub fn try_spread(&mut self) -> Option<Vec2> {
        if !self.can_spread() {
            return None;
        }

        let mut rng = rand::rng();

        // Reset timer
        self.spread_timer = self.spread_cooldown;

        // Random chance to actually spread (30% chance)
        if !rng.random_bool(0.3) {
            return None;
        }

        // Choose random direction (8 directions)
        let spread_distance = rng.random_range(15.0..30.0);
        let direction = match rng.random_range(0..8) {
            0 => Vec2::new(1.0, 0.0),   // Right
            1 => Vec2::new(-1.0, 0.0),  // Left
            2 => Vec2::new(0.0, 1.0),   // Up
            3 => Vec2::new(0.0, -1.0),  // Down
            4 => Vec2::new(1.0, 1.0),   // Top-right
            5 => Vec2::new(-1.0, 1.0),  // Top-left
            6 => Vec2::new(1.0, -1.0),  // Bottom-right
            _ => Vec2::new(-1.0, -1.0), // Bottom-left
        };

        Some(self.position + direction.normalize() * spread_distance)
    }

    pub fn is_dead(&self) -> bool {
        self.growth <= 0.0
    }
}
