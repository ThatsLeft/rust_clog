
#[derive(Debug, Clone)]
pub struct GravityField {
    pub strength: f32,
    pub radius: f32,
    pub falloff_type: GravityFalloff,
}

#[derive(Debug, Clone)]
pub enum GravityFalloff {
    Linear,
    InverseSquare,
    Constant,
    Custom(f32), // 1.0 + distance^2 * rate
}

impl GravityField {
    pub fn new(strength: f32, radius: f32, falloff_type: GravityFalloff) -> Self {
        Self { strength, radius, falloff_type }
    }
    
    pub fn calculate_force(&self, distance: f32, target_mass: f32) -> f32 {
        match self.falloff_type {
            GravityFalloff::Constant => self.strength * target_mass,
            GravityFalloff::Linear => self.strength * target_mass / distance,
            GravityFalloff::InverseSquare => self.strength * target_mass / (distance * distance),
            GravityFalloff::Custom(rate) => self.strength * target_mass / (1.0 + distance * distance * rate),
        }
    }
}