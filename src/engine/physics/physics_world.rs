use glam::Vec2;

use crate::engine::{rigid_body::{BodyId, BodyType, RigidBody}, collision::{check_collision, check_collision_with_point}};

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub body1_id: BodyId,
    pub body2_id: BodyId,
    pub contact_point: Vec2,
    pub normal: Vec2,
}

/// The main physics world that manages all physics bodies
pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
    next_body_id: u32,
    global_gravity: Vec2,
    collision_events: Vec<CollisionEvent>,

    // Performance settings
    sleep_enabled: bool,
    substeps: u32,
}

impl PhysicsWorld {
    /// Create a new physics world
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            next_body_id: 0,
            global_gravity: Vec2::ZERO,
            collision_events: Vec::new(),

            sleep_enabled: true,
            substeps: 1,
        }
    }

    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }
    
    pub fn clear_collision_events(&mut self) {
        self.collision_events.clear();
    }
    
    /// Configure gravity for the world
    pub fn set_global_gravity(&mut self, gravity: Vec2) {
        self.global_gravity = gravity;
        
        // Wake up all dynamic bodies when gravity changes
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic {
                body.wake_up();
            }
        }
    }
    
    /// Add a body to the physics world
    pub fn add_body(&mut self, mut body: RigidBody) -> BodyId {
        let id = BodyId(self.next_body_id);
        self.next_body_id += 1;
        
        body.id = id;
        self.bodies.push(body);
        
        id
    }
    
    /// Remove a body from the physics world
    pub fn remove_body(&mut self, id: BodyId) -> Option<RigidBody> {
        if let Some(index) = self.bodies.iter().position(|b| b.id == id) {
            Some(self.bodies.remove(index))
        } else {
            None
        }
    }

    pub fn remove_marked_bodies(&mut self) -> Vec<RigidBody> {
        let (remaining, removed): (Vec<_>, Vec<_>) = self.bodies
            .drain(..)
            .partition(|body| !body.marked_for_deletion);
        
        self.bodies = remaining;
        removed
    }
    
    /// Get a reference to a body
    pub fn get_body(&self, id: BodyId) -> Option<&RigidBody> {
        self.bodies.iter().find(|b| b.id == id)
    }
    
    /// Get a mutable reference to a body
    pub fn get_body_mut(&mut self, id: BodyId) -> Option<&mut RigidBody> {
        self.bodies.iter_mut().find(|b| b.id == id)
    }
    
    /// Get all bodies
    pub fn bodies(&self) -> &[RigidBody] {
        &self.bodies
    }
    
    /// Step the physics simulation forward by dt seconds
    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        
        let sub_dt = dt / self.substeps as f32;
        
        for _ in 0..self.substeps {
            self.step_internal(sub_dt);
        }
    }
    
    /// Internal physics step
    fn step_internal(&mut self, dt: f32) {
        // Apply gravity and other forces
        let bodies_clone = self.bodies.clone(); // Need clone to avoid borrow checker
    
        for i in 0..self.bodies.len() {
            if self.bodies[i].body_type == BodyType::Dynamic && !self.bodies[i].is_sleeping {
                let body = &mut self.bodies[i];
                
                // Apply global gravity
                let global_gravity_force = self.global_gravity * body.mass;
                body.force_accumulator += global_gravity_force;
                
                // Apply gravity from other bodies (if they have gravity fields)
                for other in &bodies_clone {
                    if other.id != body.id {
                        if let Some(gravity_field) = &other.gravity_field {
                            let to_other = other.position - body.position;
                            let distance_sq = to_other.length_squared();
                            let distance = distance_sq.sqrt();
                            
                            if distance < gravity_field.radius && distance > 0.1 {
                                let direction = to_other / distance;
                                let force_magnitude = gravity_field.calculate_force(distance, body.mass);
                                body.force_accumulator += direction * force_magnitude;
                            }
                        }
                    }
                }
                
                // Apply drag
                if body.material.drag > 0.0 {
                    let drag_force = -body.velocity * body.material.drag * body.mass;
                    body.force_accumulator += drag_force;
                }
            }
        }
        
        // Integrate forces and update positions
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic && !body.is_sleeping {
                // Calculate acceleration from forces (F = ma, so a = F/m)
                body.acceleration = body.force_accumulator / body.mass;
                
                // Integrate velocity (v = v0 + a*dt)
                body.velocity += body.acceleration * dt;
                
                // Integrate position (x = x0 + v*dt)
                body.position += body.velocity * dt;
                
                // Update collider position
                body.collider.position = body.position;
                
                // Clear force accumulator for next frame
                body.force_accumulator = Vec2::ZERO;
                
                // Update sleep timer
                if self.sleep_enabled {
                    const SLEEP_VELOCITY_THRESHOLD: f32 = 0.1;
                    
                    if body.velocity.length() < SLEEP_VELOCITY_THRESHOLD {
                        body.sleep_timer += dt;
                    } else {
                        body.sleep_timer = 0.0;
                    }
                    
                    // Put body to sleep if it's been still long enough
                    if body.should_sleep() {
                        body.is_sleeping = true;
                        body.velocity = Vec2::ZERO;
                    }
                }
            } else if body.body_type == BodyType::Kinematic {
                // Kinematic bodies only update position based on velocity
                body.position += body.velocity * dt;
                body.collider.position = body.position;
            }
        }
        
        // Check for collisions and resolve them
        self.resolve_collisions();
    }
    
    /// Set the number of physics substeps (higher = more accurate but slower)
    pub fn set_substeps(&mut self, substeps: u32) {
        self.substeps = substeps.max(1);
    }
    
    /// Enable or disable sleeping (performance optimization)
    pub fn set_sleep_enabled(&mut self, enabled: bool) {
        self.sleep_enabled = enabled;
        
        if !enabled {
            // Wake up all sleeping bodies
            for body in &mut self.bodies {
                body.wake_up();
            }
        }
    }
    
    /// Check for collisions between all bodies and resolve them
    fn resolve_collisions(&mut self) {
        
        // Collect collision pairs first to avoid borrowing issues
        let mut collision_pairs = Vec::new();
        
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                // Skip collision between static bodies
                if self.bodies[i].body_type == BodyType::Static && self.bodies[j].body_type == BodyType::Static {
                    continue;
                }
                
                // Check if bodies are colliding
                if check_collision(&self.bodies[i].collider, &self.bodies[j].collider) {
                    collision_pairs.push((i, j));
                }
            }
        }
        
        // Resolve collisions
        for (i, j) in collision_pairs {
            self.resolve_collision_pair(i, j);
        }
    }
    
    /// Resolve collision between two bodies by index
    fn resolve_collision_pair(&mut self, i: usize, j: usize) {
        // Get collision details
        let collision_result = check_collision_with_point(&self.bodies[i].collider, &self.bodies[j].collider);
        
        if !collision_result.collided {
            return;
        }
        
        // Calculate collision normal and penetration depth
        let normal = (self.bodies[j].position - self.bodies[i].position).normalize();
        self.collision_events.push(CollisionEvent {
            body1_id: self.bodies[i].id,
            body2_id: self.bodies[j].id,
            contact_point: collision_result.contact_point,
            normal,
        });
        
        // Calculate relative velocity
        let relative_velocity = self.bodies[j].velocity - self.bodies[i].velocity;
        let velocity_along_normal = relative_velocity.dot(normal);
        
        // Don't resolve if velocities are separating
        if velocity_along_normal > 0.0 {
            return;
        }
        
        // Calculate restitution (bounciness)
        let restitution = (self.bodies[i].material.restitution + self.bodies[j].material.restitution) / 2.0;
        
        // Calculate impulse scalar
        let impulse_scalar = -(1.0 + restitution) * velocity_along_normal;
        
        // Handle infinite mass (static bodies)
        let inv_mass1 = if self.bodies[i].mass.is_infinite() { 0.0 } else { 1.0 / self.bodies[i].mass };
        let inv_mass2 = if self.bodies[j].mass.is_infinite() { 0.0 } else { 1.0 / self.bodies[j].mass };
        
        let impulse_scalar = impulse_scalar / (inv_mass1 + inv_mass2);
        let impulse = normal * impulse_scalar;
        
        // Apply impulse
        if self.bodies[i].body_type == BodyType::Dynamic {
            self.bodies[i].velocity -= impulse * inv_mass1;
        }
        if self.bodies[j].body_type == BodyType::Dynamic {
            self.bodies[j].velocity += impulse * inv_mass2;
        }
        
        // Position correction to prevent sinking
        const CORRECTION_PERCENT: f32 = 0.2;
        const CORRECTION_SLOP: f32 = 0.01;
        
        let penetration = self.calculate_penetration(&self.bodies[i].collider, &self.bodies[j].collider);
        if penetration > CORRECTION_SLOP {
            let correction = normal * penetration * CORRECTION_PERCENT / (inv_mass1 + inv_mass2);
            
            if self.bodies[i].body_type == BodyType::Dynamic {
                self.bodies[i].position -= correction * inv_mass1;
                self.bodies[i].collider.position = self.bodies[i].position;
            }
            if self.bodies[j].body_type == BodyType::Dynamic {
                self.bodies[j].position += correction * inv_mass2;
                self.bodies[j].collider.position = self.bodies[j].position;
            }
        }
    }
    
    
    /// Calculate penetration depth between two colliders
    fn calculate_penetration(&self, collider1: &crate::engine::Collider, collider2: &crate::engine::Collider) -> f32 {
        use crate::engine::CollisionShape;
        
        match (&collider1.shape, &collider2.shape) {
            (CollisionShape::Circle { radius: r1 }, CollisionShape::Circle { radius: r2 }) => {
                let distance = (collider1.position - collider2.position).length();
                (r1 + r2) - distance
            },
            (CollisionShape::Rectangle { width: w1, height: h1 }, CollisionShape::Circle { radius: r2 }) => {
                // Convert rectangle from center position to min/max bounds
                let rect_min = Vec2::new(collider1.position.x - w1 / 2.0, collider1.position.y - h1 / 2.0);
                let rect_max = Vec2::new(collider1.position.x + w1 / 2.0, collider1.position.y + h1 / 2.0);
                let closest_x = collider2.position.x.max(rect_min.x).min(rect_max.x);
                let closest_y = collider2.position.y.max(rect_min.y).min(rect_max.y);
                let distance = (collider2.position - Vec2::new(closest_x, closest_y)).length();
                r2 - distance
            },
            (CollisionShape::Circle { radius: r1 }, CollisionShape::Rectangle { width: w2, height: h2 }) => {
                // Convert rectangle from center position to min/max bounds
                let rect_min = Vec2::new(collider2.position.x - w2 / 2.0, collider2.position.y - h2 / 2.0);
                let rect_max = Vec2::new(collider2.position.x + w2 / 2.0, collider2.position.y + h2 / 2.0);
                let closest_x = collider1.position.x.max(rect_min.x).min(rect_max.x);
                let closest_y = collider1.position.y.max(rect_min.y).min(rect_max.y);
                let distance = (collider1.position - Vec2::new(closest_x, closest_y)).length();
                r1 - distance
            },
            (CollisionShape::Rectangle { width: w1, height: h1 }, CollisionShape::Rectangle { width: w2, height: h2 }) => {
                // Convert rectangles from center position to min/max bounds
                let min1 = Vec2::new(collider1.position.x - w1 / 2.0, collider1.position.y - h1 / 2.0);
                let max1 = Vec2::new(collider1.position.x + w1 / 2.0, collider1.position.y + h1 / 2.0);
                let min2 = Vec2::new(collider2.position.x - w2 / 2.0, collider2.position.y - h2 / 2.0);
                let max2 = Vec2::new(collider2.position.x + w2 / 2.0, collider2.position.y + h2 / 2.0);
                
                // Calculate overlap for AABB
                let overlap_x = max1.x.min(max2.x) - min1.x.max(min2.x);
                let overlap_y = max1.y.min(max2.y) - min1.y.max(min2.y);
                overlap_x.min(overlap_y)
            },
        }
    }

    /// Get physics world statistics
    pub fn stats(&self) -> PhysicsStats {
        let total_bodies = self.bodies.len();
        let sleeping_bodies = self.bodies.iter().filter(|b| b.is_sleeping).count();
        let total_energy = self.bodies.iter().map(|b| b.kinetic_energy()).sum();
        
        PhysicsStats {
            total_bodies,
            active_bodies: total_bodies - sleeping_bodies,
            sleeping_bodies,
            total_kinetic_energy: total_energy,
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Physics world statistics for debugging
#[derive(Debug, Clone)]
pub struct PhysicsStats {
    pub total_bodies: usize,
    pub active_bodies: usize,
    pub sleeping_bodies: usize,
    pub total_kinetic_energy: f32,
}