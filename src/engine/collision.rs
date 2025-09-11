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

#[derive(Copy, Clone, Debug)]
pub struct CollisionResult {
    pub collided: bool,
    pub contact_point: Vec2,
}

impl CollisionResult {
    pub fn none() -> Self {
        Self { collided: false, contact_point: Vec2::ZERO }
    }
    
    pub fn hit(point: Vec2) -> Self {
        Self { collided: true, contact_point: point }
    }
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

pub fn check_collision_with_point(a: &Collider, b: &Collider) -> CollisionResult {
    match (&a.shape, &b.shape) {
        (CollisionShape::Rectangle { width: w1, height: h1 },
         CollisionShape::Rectangle { width: w2, height: h2 }) => {
            aabb_vs_aabb_with_point(a.position, *w1, *h1, b.position, *w2, *h2)
        },
        (CollisionShape::Circle { radius: r1 },
         CollisionShape::Circle { radius: r2 }) => {
            circle_vs_circle_with_point(a.position, *r1, b.position, *r2)
        },
        (CollisionShape::Rectangle { width, height }, 
         CollisionShape::Circle { radius }) => {
            aabb_vs_circle_with_point(a.position, *width, *height, b.position, *radius)
        },
        (CollisionShape::Circle { radius }, 
         CollisionShape::Rectangle { width, height }) => {
            aabb_vs_circle_with_point(b.position, *width, *height, a.position, *radius)
        },
    }
}

fn aabb_vs_aabb(pos1: Vec2, w1: f32, h1: f32, pos2: Vec2, w2: f32, h2: f32) -> bool {
    // Convert from center position to min/max bounds
    let min1 = Vec2::new(pos1.x - w1 / 2.0, pos1.y - h1 / 2.0);
    let max1 = Vec2::new(pos1.x + w1 / 2.0, pos1.y + h1 / 2.0);
    let min2 = Vec2::new(pos2.x - w2 / 2.0, pos2.y - h2 / 2.0);
    let max2 = Vec2::new(pos2.x + w2 / 2.0, pos2.y + h2 / 2.0);
    
    (min1.x < max2.x) && 
    (max1.x > min2.x) && 
    (min1.y < max2.y) && 
    (max1.y > min2.y)
}

fn circle_vs_circle(pos1: Vec2, r1: f32, pos2: Vec2, r2: f32) -> bool {
    let distance_sq = (pos1 - pos2).length_squared();
    let radius_sum = r1 + r2;
    distance_sq <= radius_sum * radius_sum
}

fn aabb_vs_circle(rect_pos: Vec2, width: f32, height: f32, circle_pos: Vec2, radius: f32) -> bool {
    // Convert rectangle from center position to min/max bounds
    let rect_min = Vec2::new(rect_pos.x - width / 2.0, rect_pos.y - height / 2.0);
    let rect_max = Vec2::new(rect_pos.x + width / 2.0, rect_pos.y + height / 2.0);
    
    let closest_x = circle_pos.x.max(rect_min.x).min(rect_max.x);
    let closest_y = circle_pos.y.max(rect_min.y).min(rect_max.y);
    let distance_sq = (circle_pos - Vec2::new(closest_x, closest_y)).length_squared();
    distance_sq <= radius * radius
}

fn aabb_vs_aabb_with_point(pos1: Vec2, w1: f32, h1: f32, pos2: Vec2, w2: f32, h2: f32) -> CollisionResult {
    // Convert from center position to min/max bounds
    let min1 = Vec2::new(pos1.x - w1 / 2.0, pos1.y - h1 / 2.0);
    let max1 = Vec2::new(pos1.x + w1 / 2.0, pos1.y + h1 / 2.0);
    let min2 = Vec2::new(pos2.x - w2 / 2.0, pos2.y - h2 / 2.0);
    let max2 = Vec2::new(pos2.x + w2 / 2.0, pos2.y + h2 / 2.0);
    
    let collided = (min1.x < max2.x) &&
                   (max1.x > min2.x) &&
                   (min1.y < max2.y) &&
                   (max1.y > min2.y);
    
    if collided {
        // Calculate overlap region center
        let left = min1.x.max(min2.x);
        let right = max1.x.min(max2.x);
        let top = min1.y.max(min2.y);
        let bottom = max1.y.min(max2.y);
        
        let contact_point = Vec2::new((left + right) * 0.5, (top + bottom) * 0.5);
        CollisionResult::hit(contact_point)
    } else {
        CollisionResult::none()
    }
}

fn circle_vs_circle_with_point(pos1: Vec2, r1: f32, pos2: Vec2, r2: f32) -> CollisionResult {
    let distance_sq = (pos1 - pos2).length_squared();
    let radius_sum = r1 + r2;
    let collided = distance_sq <= radius_sum * radius_sum;
    
    if collided {
        // Contact point is along the line between centers
        let direction = (pos2 - pos1).normalize();
        let contact_point = pos1 + direction * r1;
        CollisionResult::hit(contact_point)
    } else {
        CollisionResult::none()
    }
}

fn aabb_vs_circle_with_point(rect_pos: Vec2, width: f32, height: f32, circle_pos: Vec2, radius: f32) -> CollisionResult {
    // Convert rectangle from center position to min/max bounds
    let rect_min = Vec2::new(rect_pos.x - width / 2.0, rect_pos.y - height / 2.0);
    let rect_max = Vec2::new(rect_pos.x + width / 2.0, rect_pos.y + height / 2.0);
    
    let closest_x = circle_pos.x.max(rect_min.x).min(rect_max.x);
    let closest_y = circle_pos.y.max(rect_min.y).min(rect_max.y);
    let closest_point = Vec2::new(closest_x, closest_y);
    let distance_sq = (circle_pos - closest_point).length_squared();
    let collided = distance_sq <= radius * radius;
    
    if collided {
        CollisionResult::hit(closest_point)
    } else {
        CollisionResult::none()
    }
}