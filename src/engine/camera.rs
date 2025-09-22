use std::hash::{DefaultHasher, Hash, Hasher};

use glam::{Mat4, Vec2};

pub struct Camera2D {
    pub position: Vec2, // World position the camera is looking at
    pub zoom: f32,      // Zoom level (1.0 = normal, 2.0 = zoomed in 2x)
    pub rotation: f32,  // Camera rotation in radians

    shake_offset: Vec2,
    shake_intensity: f32,
    shake_duration: f32,
    shake_timer: f32,

    view_projection: Mat4,

    // Internal state
    transform_dirty: bool,
    viewport_width: f32,
    viewport_height: f32,
}

/// Engine functions for camera
impl Camera2D {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            shake_offset: Vec2::ZERO,
            shake_intensity: 0.0,
            shake_duration: 0.0,
            shake_timer: 0.0,
            view_projection: Mat4::IDENTITY,
            transform_dirty: true,
            viewport_width: 800.0, // Default size
            viewport_height: 600.0,
        }
    }

    // Engine calls this each frame
    pub fn update_shake(&mut self, dt: f32) {
        if self.shake_timer > 0.0 {
            self.shake_timer -= dt;

            // Calculate shake strength (fades out over time)
            let shake_strength = (self.shake_timer / self.shake_duration) * self.shake_intensity;

            // Random shake offset
            let mut hasher = DefaultHasher::new();
            let shake_timer = (self.shake_timer as f32 * 1000.0) as u32;
            shake_timer.hash(&mut hasher);
            self.position.x.to_bits().hash(&mut hasher);
            let hash = hasher.finish();

            let random_angle = ((hash & 0xFFFF) as f32 / 65535.0) * 2.0 * std::f32::consts::PI;
            self.shake_offset = Vec2::new(
                random_angle.cos() * shake_strength,
                random_angle.sin() * shake_strength,
            );

            self.transform_dirty = true; // Need to recalculate matrix
        } else {
            // No more shake
            if self.shake_offset != Vec2::ZERO {
                self.shake_offset = Vec2::ZERO;
                self.transform_dirty = true;
            }
        }
    }

    // Engine calls this when window size changes
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        if self.viewport_width != width || self.viewport_height != height {
            self.viewport_width = width;
            self.viewport_height = height;
            self.transform_dirty = true;
        }
    }

    // Engine calls this to get the matrix for rendering
    pub fn get_view_projection_matrix(&mut self) -> Mat4 {
        if self.transform_dirty {
            self.update_matrices();
        }
        self.view_projection
    }

    // Internal matrix calculation
    fn update_matrices(&mut self) {
        // Create orthographic projection (maps world space to clip space)
        let half_width = self.viewport_width * 0.5 / self.zoom;
        let half_height = self.viewport_height * 0.5 / self.zoom;

        let projection = Mat4::orthographic_rh(
            -half_width,
            half_width, // left, right
            -half_height,
            half_height, // bottom, top (flipped for screen space)
            -1.0,
            1.0, // near, far
        );

        // Create view matrix (camera transform)
        let effective_position = self.position + self.shake_offset; // ADD shake offset
        let translation = Mat4::from_translation(glam::Vec3::new(
            -effective_position.x,
            -effective_position.y,
            0.0,
        ));
        let rotation = Mat4::from_rotation_z(-self.rotation);
        let view = rotation * translation;

        // Combine into view-projection matrix
        self.view_projection = projection * view;
        self.transform_dirty = false;
    }
}

impl Camera2D {
    // Coordinate conversion methods (useful for mouse interaction)
    pub fn screen_to_world(&mut self, screen_pos: Vec2) -> Vec2 {
        if self.transform_dirty {
            self.update_matrices();
        }

        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc_x = (screen_pos.x / self.viewport_width) * 2.0 - 1.0;
        let ndc_y = -((screen_pos.y / self.viewport_height) * 2.0 - 1.0); // ADD negative sign here

        // Transform by inverse view-projection matrix
        let inverse_vp = self.view_projection.inverse();
        let world_pos_4d = inverse_vp * glam::Vec4::new(ndc_x, ndc_y, 0.0, 1.0);

        Vec2::new(world_pos_4d.x, world_pos_4d.y)
    }

    pub fn world_to_screen(&mut self, world_pos: Vec2) -> Vec2 {
        if self.transform_dirty {
            self.update_matrices();
        }

        // Transform world position by view-projection matrix
        let clip_pos = self.view_projection * glam::Vec4::new(world_pos.x, world_pos.y, 0.0, 1.0);

        // Convert from normalized device coordinates to screen coordinates
        let screen_x = (clip_pos.x + 1.0) * 0.5 * self.viewport_width;
        let screen_y = (clip_pos.y + 1.0) * 0.5 * self.viewport_height;

        Vec2::new(screen_x, screen_y)
    }

    pub fn set_position(&mut self, position: Vec2) {
        if self.position != position {
            self.position = position;
            self.transform_dirty = true;
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        let clamped_zoom = zoom.max(0.1).min(10.0); // Reasonable zoom limits
        if self.zoom != clamped_zoom {
            self.zoom = clamped_zoom;
            self.transform_dirty = true;
        }
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        if self.rotation != rotation {
            self.rotation = rotation;
            self.transform_dirty = true;
        }
    }

    // Game calls this to trigger shake
    pub fn add_shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
        self.shake_timer = duration;
    }

    // Camera movement methods
    pub fn move_by(&mut self, delta: Vec2) {
        self.set_position(self.position + delta);
    }

    pub fn zoom_by(&mut self, delta: f32) {
        self.set_zoom(self.zoom + delta);
    }

    pub fn rotate_by(&mut self, delta: f32) {
        self.set_rotation(self.rotation + delta);
    }

    // Query methods
    pub fn get_position(&self) -> Vec2 {
        self.position
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    pub fn view_half_extents(&self) -> Vec2 {
        Vec2::new(
            self.viewport_width * 0.5 / self.zoom,
            self.viewport_height * 0.5 / self.zoom,
        )
    }

    pub fn clamp_to_bounds(&mut self, min: Vec2, max: Vec2) {
        let half = self.view_half_extents();

        // Only clamp if the world is larger than the view area
        let clamp_min_x = min.x + half.x;
        let clamp_max_x = max.x - half.x;
        let clamp_min_y = min.y + half.y;
        let clamp_max_y = max.y - half.y;

        let clamped_x = if clamp_min_x <= clamp_max_x {
            self.position.x.clamp(clamp_min_x, clamp_max_x)
        } else {
            // View is larger than world bounds, center camera
            (min.x + max.x) * 0.5
        };

        let clamped_y = if clamp_min_y <= clamp_max_y {
            self.position.y.clamp(clamp_min_y, clamp_max_y)
        } else {
            // View is larger than world bounds, center camera
            (min.y + max.y) * 0.5
        };

        self.set_position(Vec2::new(clamped_x, clamped_y));
    }

    pub fn visible_aabb(&self) -> (Vec2, Vec2) {
        let half = self.view_half_extents();
        (self.position - half, self.position + half)
    }
}
