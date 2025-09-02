use glam::Vec2;

#[derive(Copy, Clone, Debug)]
pub enum CollisionShape {
    Rectangle { width: f32, height: f32 },
    Circle { radius: f32 },
}

#[derive(Copy, Clone, Debug)]
pub struct Collider {
    pub position: Vec2,
    pub shape: CollisionShape,
    pub is_trigger: bool,  // If true, detects collision but doesn't block movement
}

impl Collider {
    pub fn new_rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            shape: CollisionShape::Rectangle { width, height },
            is_trigger: false,
        }
    }
    
    pub fn new_circle(x: f32, y: f32, radius: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            shape: CollisionShape::Circle { radius },
            is_trigger: false,
        }
    }
    
}

pub fn check_collision(a: &Collider, b: &Collider) -> bool {
    match (&a.shape, &b.shape) {
        (CollisionShape::Rectangle { width: w1, height: h1 }, 
         CollisionShape::Rectangle { width: w2, height: h2 }) => {
            aabb_vs_aabb(a.position, *w1, *h1, b.position, *w2, *h2)
        },
        (CollisionShape::Circle { radius: r1 }, 
         CollisionShape::Circle { radius: r2 }) => {
            circle_vs_circle(a.position, *r1, b.position, *r2)
        },
        (CollisionShape::Rectangle { width, height }, 
         CollisionShape::Circle { radius }) => {
            aabb_vs_circle(a.position, *width, *height, b.position, *radius)
        },
        (CollisionShape::Circle { radius }, 
         CollisionShape::Rectangle { width, height }) => {
            aabb_vs_circle(b.position, *width, *height, a.position, *radius)
        },
    }
}

fn aabb_vs_aabb(pos1: Vec2, w1: f32, h1: f32, pos2: Vec2, w2: f32, h2: f32) -> bool {
    (pos1.x < pos2.x + w2) && 
    (pos1.x + w1 > pos2.x) && 
    (pos1.y < pos2.y + h2) && 
    (pos1.y + h1 > pos2.y)
}

fn circle_vs_circle(pos1: Vec2, r1: f32, pos2: Vec2, r2: f32) -> bool {
    let distance_sq = (pos1 - pos2).length_squared();
    let radius_sum = r1 + r2;
    distance_sq <= radius_sum * radius_sum
}

fn aabb_vs_circle(rect_pos: Vec2, width: f32, height: f32, circle_pos: Vec2, radius: f32) -> bool {
    let closest_x = circle_pos.x.max(rect_pos.x).min(rect_pos.x + width);
    let closest_y = circle_pos.y.max(rect_pos.y).min(rect_pos.y + height);
    let distance_sq = (circle_pos - Vec2::new(closest_x, closest_y)).length_squared();
    distance_sq <= radius * radius
}