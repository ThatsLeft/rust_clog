use glam::{Vec2, Vec4};

use crate::engine::{Renderer, Sprite};

#[derive(Clone)]
pub struct TextRenderer {
    texture_name: String,
    glyph_size: Vec2, // glyph size in pixels in the source atlas (e.g. 8x8 or 16x16)
    atlas_cols: u32,  // e.g. 16
    atlas_rows: u32,  // e.g. 16
    color: Vec4,      // default color
    scale: f32,       // default scale
    spacing: f32,     // extra advance between glyphs (in source glyph pixels)
    first_codepoint: u32,
}

impl TextRenderer {
    pub fn new(
        texture_name: &str,
        glyph_w: f32,
        glyph_h: f32,
        atlas_cols: u32,
        atlas_rows: u32,
    ) -> Self {
        Self {
            texture_name: texture_name.to_string(),
            glyph_size: Vec2::new(glyph_w, glyph_h),
            atlas_cols,
            atlas_rows,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            scale: 1.0,
            spacing: 0.0,
            first_codepoint: 32,
        }
    }

    pub fn set_color(&mut self, color: Vec4) {
        self.color = color;
    }
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.01);
    }
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }

    pub fn measure_single_line_px(&self, text: &str) -> Vec2 {
        let w = (text.chars().count() as f32) * (self.glyph_size.x + self.spacing) * self.scale;
        let h = self.glyph_size.y * self.scale;
        Vec2::new(w, h)
    }

    // Draw anchored in world space (respects camera)
    pub fn draw_text_world(&self, renderer: &mut Renderer, mut pos: Vec2, text: &str) {
        let uv_w = 1.0 / self.atlas_cols as f32;
        let uv_h = 1.0 / self.atlas_rows as f32;

        let adv_x = (self.glyph_size.x + self.spacing) * self.scale;
        let adv_y = (self.glyph_size.y + self.spacing) * self.scale;

        let line_start_x = pos.x;

        for ch in text.chars() {
            if ch == '\n' {
                pos.x = line_start_x;
                pos.y -= adv_y;
                continue;
            }

            // Map from Unicode codepoint to atlas index starting at first_codepoint (' ' = 32)
            let code = ch as u32;
            let idx = code.saturating_sub(self.first_codepoint);
            if idx >= (self.atlas_cols * self.atlas_rows) {
                pos.x += adv_x;
                continue;
            }

            let col = (idx % self.atlas_cols) as f32;
            let row = (idx / self.atlas_cols) as f32;

            // Flip V so row 0 is the top row of the bitmap
            let u = col * uv_w;
            let v = row * uv_h;

            let uv = glam::Vec4::new(u, v, uv_w, uv_h);

            let mut sprite = Sprite::new()
                .with_texture_name(self.texture_name.clone())
                .with_position(pos + self.glyph_size * 0.5 * self.scale)
                .with_size(self.glyph_size * self.scale)
                .with_uv(uv)
                .with_color(self.color)
                .with_flip_y(true);

            renderer.draw_sprite(&mut sprite);
            pos.x += adv_x;
        }
    }

    // Draw at screen pixel position (top-left origin) regardless of camera
    pub fn draw_text_screen(
        &self,
        renderer: &mut Renderer,
        camera: &mut crate::engine::Camera2D,
        screen_pos: Vec2,
        text: &str,
    ) {
        // Convert top-left screen pixels to world. Current camera has center-origin projection,
        // so convert the intended screen-space anchor to world coords.
        let world = camera.screen_to_world(screen_pos);
        self.draw_text_world(renderer, world, text);
    }

    // Helpers for common placements (top-left etc.) at pixel offsets
    pub fn draw_top_left(
        &self,
        renderer: &mut Renderer,
        camera: &mut crate::engine::Camera2D,
        offset_px: Vec2,
        text: &str,
    ) {
        self.draw_text_screen(renderer, camera, offset_px, text);
    }

    pub fn draw_top_right(
        &self,
        renderer: &mut Renderer,
        camera: &mut crate::engine::Camera2D,
        offset_px: Vec2,
        text: &str,
    ) {
        let size = self.measure_single_line_px(text);
        let x = sokol::app::width() as f32 - offset_px.x - size.x;
        let y = offset_px.y;
        self.draw_text_screen(renderer, camera, Vec2::new(x, y), text);
    }
}
