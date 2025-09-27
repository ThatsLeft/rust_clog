use crate::engine::{gravity::GravityField, world_bounds::BoundsBehavior, Collider};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsMaterial {
    /// How bouncy the object is (0.0 = no bounce, 1.0 = perfect bounce)
    pub restitution: f32,
    /// Surface friction coefficient (0.0 = no friction, 1.0 = high friction)
    pub friction: f32,
    /// Air resistance (0.0 = no drag, higher values = more drag)
    pub drag: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            restitution: 0.0,
            friction: 0.5,
            drag: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub id: BodyId,

    pub body_type: BodyType,

    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,

    pub mass: f32,

    pub material: PhysicsMaterial,
    pub collider: Collider,
    pub gravity_field: Option<GravityField>,
    pub marked_for_deletion: bool,

    pub rotation: f32,
    pub angular_velocity: f32,
    pub angular_acceleration: f32,
    pub moment_of_inertia: f32,

    pub bounds_behavior: Option<BoundsBehavior>,

    // Internal state
    pub(crate) torque_accumulator: f32,
    pub(crate) force_accumulator: Vec2,
    pub(crate) is_sleeping: bool,
    pub(crate) sleep_timer: f32,
}

impl RigidBody {
    /// Create a new dynamic rigid body
    pub fn new_dynamic(id: BodyId, position: Vec2, collider: Collider, mass: f32) -> Self {
        let moment_of_inertia = Self::calculate_moment_of_inertia(&collider, mass);

        Self {
            id,
            body_type: BodyType::Dynamic,
            position,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            mass: mass.max(0.001), // Prevent division by zero
            material: PhysicsMaterial::default(),
            collider,
            gravity_field: None,
            marked_for_deletion: false,

            rotation: 0.0,
            angular_velocity: 0.0,
            angular_acceleration: 0.0,
            moment_of_inertia,

            bounds_behavior: None,

            torque_accumulator: 0.0,
            force_accumulator: Vec2::ZERO,
            is_sleeping: false,
            sleep_timer: 0.0,
        }
    }

    /// Create a new static rigid body (walls, platforms)
    pub fn new_static(id: BodyId, position: Vec2, collider: Collider) -> Self {
        let moment_of_inertia = Self::calculate_moment_of_inertia(&collider, f32::INFINITY);

        Self {
            id,
            body_type: BodyType::Static,
            position,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            mass: f32::INFINITY,
            material: PhysicsMaterial::default(),
            collider,
            gravity_field: None,
            marked_for_deletion: false,

            rotation: 0.0,
            angular_velocity: 0.0,
            angular_acceleration: 0.0,
            moment_of_inertia,

            bounds_behavior: Some(BoundsBehavior::Ignore),

            torque_accumulator: 0.0,
            force_accumulator: Vec2::ZERO,
            is_sleeping: true, // Static bodies are always "sleeping"
            sleep_timer: 0.0,
        }
    }

    /// Create a new kinematic rigid body (moving platforms)
    pub fn new_kinematic(id: BodyId, position: Vec2, collider: Collider) -> Self {
        let moment_of_inertia = Self::calculate_moment_of_inertia(&collider, f32::INFINITY);

        Self {
            id,
            body_type: BodyType::Kinematic,
            position,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            mass: f32::INFINITY,
            material: PhysicsMaterial::default(),
            collider,
            gravity_field: None,
            marked_for_deletion: false,

            rotation: 0.0,
            angular_velocity: 0.0,
            angular_acceleration: 0.0,
            moment_of_inertia,

            bounds_behavior: None,

            torque_accumulator: 0.0,
            force_accumulator: Vec2::ZERO,
            is_sleeping: false,
            sleep_timer: 0.0,
        }
    }

    pub fn with_bounds_behavior(mut self, behavior: BoundsBehavior) -> Self {
        self.bounds_behavior = Some(behavior);
        self
    }

    fn calculate_moment_of_inertia(collider: &Collider, mass: f32) -> f32 {
        use crate::engine::CollisionShape;

        match &collider.shape {
            CollisionShape::Circle { radius } => {
                // Solid disk: I = (1/2) * m * r²
                0.5 * mass * radius * radius
            }
            CollisionShape::Rectangle { width, height } => {
                // Solid rectangle: I = (1/12) * m * (w² + h²)
                mass * (width * width + height * height) / 12.0
            }
        }
    }

    pub fn mark_for_deletion(&mut self) {
        self.marked_for_deletion = true;
    }

    /// Apply torque (rotational force)
    pub fn apply_torque(&mut self, torque: f32) {
        if self.body_type == BodyType::Dynamic {
            self.torque_accumulator += torque;
            self.wake_up();
        }
    }

    /// Apply angular impulse (instant angular velocity change)
    pub fn apply_angular_impulse(&mut self, impulse: f32) {
        if self.body_type == BodyType::Dynamic {
            self.angular_velocity += impulse / self.moment_of_inertia;
            self.wake_up();
        }
    }

    /// Apply a force to this body (will be integrated next physics step)
    pub fn apply_force(&mut self, force: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.force_accumulator += force;
            self.wake_up();
        }
    }

    /// Apply an impulse (instant velocity change)
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse / self.mass;
            self.wake_up();
        }
    }

    /// Set velocity directly (useful for kinematic bodies)
    pub fn set_velocity(&mut self, velocity: Vec2) {
        if self.body_type != BodyType::Static {
            self.velocity = velocity;
            if self.body_type == BodyType::Dynamic {
                self.wake_up();
            }
        }
    }

    /// Set position directly
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.collider.position = position;
        if self.body_type == BodyType::Dynamic {
            self.wake_up();
        }
    }

    /// Wake up the body (stop it from sleeping)
    pub fn wake_up(&mut self) {
        if self.body_type == BodyType::Dynamic {
            self.is_sleeping = false;
            self.sleep_timer = 0.0;
        }
    }

    /// Check if the body should go to sleep (performance optimization)
    pub fn should_sleep(&self) -> bool {
        const SLEEP_VELOCITY_THRESHOLD: f32 = 0.1;
        const SLEEP_TIME_THRESHOLD: f32 = 1.0;

        self.body_type == BodyType::Dynamic
            && self.velocity.length() < SLEEP_VELOCITY_THRESHOLD
            && self.sleep_timer > SLEEP_TIME_THRESHOLD
    }

    /// Get the current kinetic energy of the body
    pub fn kinetic_energy(&self) -> f32 {
        if self.mass.is_infinite() {
            0.0
        } else {
            0.5 * self.mass * self.velocity.length_squared()
        }
    }

    pub fn clear_forces(&mut self) {
        self.force_accumulator = Vec2::ZERO;
    }
}

/// Builder pattern for useful properties
impl RigidBody {
    /// Set the initial position
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Set the initial velocity
    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set the initial acceleration
    pub fn with_acceleration(mut self, acceleration: Vec2) -> Self {
        self.acceleration = acceleration;
        self
    }

    /// Set the mass (clamped to a minimum to avoid division by zero)
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass.max(0.001);
        self
    }

    /// Replace the collider
    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.collider = collider;
        self
    }

    /// Replace the full physics material
    pub fn with_material(mut self, material: PhysicsMaterial) -> Self {
        self.material = material;
        self
    }

    /// Convenience: set restitution on the material
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.material.restitution = restitution;
        self
    }

    /// Convenience: set friction on the material
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.material.friction = friction;
        self
    }

    /// Convenience: set drag on the material
    pub fn with_drag(mut self, drag: f32) -> Self {
        self.material.drag = drag;
        self
    }

    pub fn with_gravity_field(mut self, gravity_field: GravityField) -> Self {
        self.gravity_field = Some(gravity_field);
        self
    }

    /// Add a gravity field to an existing body
    pub fn set_gravity_field(&mut self, gravity_field: Option<GravityField>) {
        self.gravity_field = gravity_field;
    }
}
