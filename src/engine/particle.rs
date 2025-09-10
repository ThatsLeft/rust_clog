use std::default;

use glam::{Vec2, Vec4};
use rand::{rng, Rng};

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Vec4,
}

#[derive(Clone)]
pub enum ParticleColorSpec {
    Fixed(Vec4),
    Range {min: Vec4, max: Vec4},
    Palette(Vec<Vec4>),
}

impl ParticleColorSpec {
    fn default() -> Self {
        Self::Fixed(Vec4::new(1.0, 1.0, 1.0, 1.0))
    }
}

#[derive(Clone)]
/// Specifies how a particle's initial velocity is generated when spawned.
///
/// Default is `Range { min: (-100, -100), max: (100, 100) }`, which picks
/// each component uniformly in the given ranges. Use different variants to
/// create streams, cones, or radial bursts.
pub enum ParticleVelocitySpec {
    /// Uses the same velocity vector for every spawned particle.
    /// Useful for constant streams (e.g., wind or conveyor).
    Fixed(Vec2),
    /// Samples x/y components independently and uniformly between `min` and `max`.
    /// Produces a "random-in-a-box" scatter. If any min/max are swapped, they
    /// are auto-corrected at spawn time.
    Range { min: Vec2, max: Vec2 },
    /// Emits roughly along `dir` with a random speed in [`speed_min`, `speed_max`]
    /// and a random angular deviation up to `spread_rad` (radians).
    /// Great for cones/fans like thrusters or muzzle flashes. Zero `dir`
    /// defaults to (1, 0). Speed bounds are auto-corrected and clamped ≥ 0.
    Direction { dir: Vec2, speed_min: f32, speed_max: f32, spread_rad: f32 },
    /// Chooses a random angle in [0, 2π] and a random speed in
    /// [`speed_min`, `speed_max`], emitting outward (360° burst).
    /// Ideal for explosions or omnidirectional effects. Speeds clamped ≥ 0.
    Radial { speed_min: f32, speed_max: f32 },
}

impl ParticleVelocitySpec {
    /// Returns the default velocity spec matching the engine's prior behavior:
    /// a uniform component-wise range in a centered box.
    fn default() -> Self {
        // Matches your current behavior: random in a box
        Self::Range { min: Vec2::new(-100.0, -100.0), max: Vec2::new(100.0, 100.0) }
    }
}

#[derive(Clone, PartialEq)]
pub enum ParticleSystemLifetime {
    Infinite,
    EmissionDuration,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    emission_rate: f32,
    spawn_position: Vec2,
    emission_duration: f32,
    particle_lifetime: f32,
    emission_timer: f32,
    total_time: f32,
    color_spec: ParticleColorSpec,
    velocity_spec: ParticleVelocitySpec, 
    global_accel: Vec2,                  
    drag: f32,
    lifetime: ParticleSystemLifetime                        
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
            color_spec: ParticleColorSpec::default(),
            velocity_spec: ParticleVelocitySpec::default(),
            global_accel: Vec2::ZERO,
            drag: 0.0,
            lifetime: ParticleSystemLifetime::Infinite
        }
    }

    pub fn with_fixed_color(mut self, color: Vec4) -> Self {
        self.color_spec = ParticleColorSpec::Fixed(color);
        self
    }
    
    pub fn with_color_range(mut self, min: Vec4, max: Vec4) -> Self {
        self.color_spec = ParticleColorSpec::Range { min, max };
        self
    }

    pub fn with_color_palette(mut self, palette: Vec<Vec4>) -> Self {
        self.color_spec = ParticleColorSpec::Palette(palette);
        self
    }

    pub fn with_fixed_velocity(mut self, v: Vec2) -> Self {
        self.velocity_spec = ParticleVelocitySpec::Fixed(v); self
    }

    pub fn with_velocity_range(mut self, min: Vec2, max: Vec2) -> Self {
        self.velocity_spec = ParticleVelocitySpec::Range { min, max }; self
    }

    pub fn with_velocity_direction(mut self, dir: Vec2, speed_min: f32, speed_max: f32, spread_rad: f32) -> Self {
        self.velocity_spec = ParticleVelocitySpec::Direction { dir, speed_min, speed_max, spread_rad }; self
    }

    pub fn with_velocity_radial(mut self, speed_min: f32, speed_max: f32) -> Self {
        self.velocity_spec = ParticleVelocitySpec::Radial { speed_min, speed_max }; self
    }

    pub fn with_acceleration(mut self, accel: Vec2) -> Self { 
        self.global_accel = accel; self 
    }

    pub fn with_drag(mut self, drag: f32) -> Self { 
        self.drag = drag.max(0.0); self 
    }
    
    pub fn with_lifetime(mut self, lifetime: ParticleSystemLifetime) -> Self {
        self.lifetime = lifetime;
        self
    }

    pub fn set_color_fixed(mut self, color: Vec4) -> Self {
        self.color_spec = ParticleColorSpec::Fixed(color);
        self
    }
    
    pub fn set_color_range(mut self, min: Vec4, max: Vec4) -> Self {
        self.color_spec = ParticleColorSpec::Range { min, max };
        self
    }

    pub fn set_color_palette(mut self, palette: Vec<Vec4>) -> Self {
        self.color_spec = ParticleColorSpec::Palette(palette);
        self
    }

    pub fn set_velocity_fixed(&mut self, v: Vec2) {
        self.velocity_spec = ParticleVelocitySpec::Fixed(v);
    }
    pub fn set_velocity_range(&mut self, min: Vec2, max: Vec2) {
        self.velocity_spec = ParticleVelocitySpec::Range { min, max };
    }
    pub fn set_velocity_direction(&mut self, dir: Vec2, speed_min: f32, speed_max: f32, spread_rad: f32) {
        self.velocity_spec = ParticleVelocitySpec::Direction { dir, speed_min, speed_max, spread_rad };
    }
    pub fn set_velocity_radial(&mut self, speed_min: f32, speed_max: f32) {
        self.velocity_spec = ParticleVelocitySpec::Radial { speed_min, speed_max };
    }
    pub fn set_acceleration(&mut self, accel: Vec2) { self.global_accel = accel; }
    pub fn set_drag(&mut self, drag: f32) { self.drag = drag.max(0.0); }

    pub fn set_spawn_position(&mut self, position: Vec2) { self.spawn_position = position; }

    pub fn set_emission_rate(&mut self, rate: f32) { self.emission_rate = rate.max(0.0); }

    pub fn set_lifetime(&mut self, lifetime: ParticleSystemLifetime) {
        self.lifetime = lifetime;
    }

    pub fn get_particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    pub fn is_finished(&self) -> bool {
        self.lifetime == ParticleSystemLifetime::EmissionDuration && self.total_time >= self.emission_duration
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
        if self.total_time < self.emission_duration && self.emission_rate > 0.0 {
            self.emission_timer -= dt;
            if self.emission_timer <= 0.0 {
                self.spawn_particle();
                self.emission_timer += 1.0 / self.emission_rate; // accumulate to avoid drift
            }
        } else {
            // keep timer sane when off
            
            if self.emission_rate <= 0.0 {
                self.emission_timer = 0.0;
            }
        }
    }

    fn spawn_particle(&mut self) {
        let mut rng = rand::rng();
    
        self.particles.push(Particle {
            position: self.spawn_position,
            velocity: self.next_velocity(),
            lifetime: rng.random_range(0.2..self.particle_lifetime),
            max_lifetime: self.particle_lifetime + 0.2,
            color: self.get_random_color(),
        });
    }

    fn get_random_color(&self) -> Vec4 {
        let mut rng = rand::rng();

        match &self.color_spec {
            ParticleColorSpec::Fixed(color) => *color,
            ParticleColorSpec::Range { min, max } => {
                let (rx0, rx1) = if min.x <= max.x { (min.x, max.x) } else { (max.x, min.x) };
                let (ry0, ry1) = if min.y <= max.y { (min.y, max.y) } else { (max.y, min.y) };
                let (rz0, rz1) = if min.z <= max.z { (min.z, max.z) } else { (max.z, min.z) };
                let (rw0, rw1) = if min.w <= max.w { (min.w, max.w) } else { (max.w, min.w) };

                Vec4::new(
                    rng.random_range(rx0..=rx1),
                    rng.random_range(ry0..=ry1),
                    rng.random_range(rz0..=rz1),
                    rng.random_range(rw0..=rw1),
                )
            }
            ParticleColorSpec::Palette(palette) => {
                if palette.is_empty() {
                    Vec4::new(1.0, 1.0, 1.0, 1.0)
                } else {
                    palette[rng.random_range(0..palette.len())]
                }
            },
        }
    }

    fn next_velocity(&self) -> Vec2 {
        let mut rng = rand::rng();
        match &self.velocity_spec {
            ParticleVelocitySpec::Fixed(v) => *v,
            ParticleVelocitySpec::Range { min, max } => {
                let (x0, x1) = if min.x <= max.x { (min.x, max.x) } else { (max.x, min.x) };
                let (y0, y1) = if min.y <= max.y { (min.y, max.y) } else { (max.y, min.y) };
                Vec2::new(
                    rng.random_range(x0..=x1),
                    rng.random_range(y0..=y1),
                )
            }
            ParticleVelocitySpec::Direction { dir, speed_min, speed_max, spread_rad } => {
                let base = if dir.length_squared() > 0.0 { dir.normalize() } else { Vec2::new(1.0, 0.0) };
                let (s0, s1) = if *speed_min <= *speed_max { (*speed_min, *speed_max) } else { (*speed_max, *speed_min) };
                let angle = rng.random_range((-spread_rad).min(*spread_rad)..=spread_rad.abs());
                let rot = Vec2::new(
                    base.x * angle.cos() - base.y * angle.sin(),
                    base.x * angle.sin() + base.y * angle.cos(),
                );
                let speed = rng.random_range(s0..=s1).max(0.0);
                rot * speed
            }
            ParticleVelocitySpec::Radial { speed_min, speed_max } => {
                let angle = rng.random_range(0.0..=2.0 * std::f32::consts::PI);
                let (s0, s1) = if *speed_min <= *speed_max { (*speed_min, *speed_max) } else { (*speed_max, *speed_min) };
                let speed = rng.random_range(s0..=s1).max(0.0);
                Vec2::new(angle.cos(), angle.sin()) * speed
            }
        }
    }
}