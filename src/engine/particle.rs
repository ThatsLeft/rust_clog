use glam::{Vec2, Vec4};
use rand::Rng;

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Vec4,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub emission_rate: f32,
    pub spawn_position: Vec2,
    pub emission_duration: f32,
    pub particle_lifetime: f32,
    emission_timer: f32,
    total_time: f32
}

impl ParticleSystem {
    pub fn new(spawn_position: Vec2, emission_rate: f32, emission_duration: f32, particle_lifetime: f32,) -> Self {
        Self {
            particles: Vec::new(),
            emission_rate,
            spawn_position,
            emission_duration,
            particle_lifetime,
            total_time: 0.0,
            emission_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.total_time += dt;

        // Update existing particles
        for particle in &mut self.particles {
            particle.position += particle.velocity * dt;
            particle.lifetime -= dt;
        }
        
        // Remove dead particles
        self.particles.retain(|p| p.lifetime > 0.0);
        
        // Spawn new particles only if within emission duration
        if self.total_time < self.emission_duration {
            self.emission_timer -= dt;
            if self.emission_timer <= 0.0 {  // CHANGE: <= instead of >=
                self.spawn_particle();
                self.emission_timer = 1.0 / self.emission_rate;  // Reset timer
            }
        }
    }

    fn spawn_particle(&mut self) {
        let mut rng = rand::rng();
    
        self.particles.push(Particle {
            position: self.spawn_position,
            velocity: Vec2::new(
                rng.random_range(-100.0..100.0),
                rng.random_range(-100.0..100.0),
            ),
            lifetime: rng.random_range(0.2..self.particle_lifetime),
            max_lifetime: self.particle_lifetime + 0.2,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        });
    }
}