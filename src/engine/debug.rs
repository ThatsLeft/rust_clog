// src/engine/debug.rs

use glam::Vec2;
use sokol::{app as sapp, debugtext as sdtx};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct DebugFlags {
    pub debug_text: AtomicBool,
    pub collision: AtomicBool,
    pub show_debug_panel: AtomicBool,
}

impl DebugFlags {
    pub fn new() -> Self {
        Self {
            debug_text: AtomicBool::new(false),
            collision: AtomicBool::new(false),
            show_debug_panel: AtomicBool::new(false),
        }
    }

    pub fn set_debug_text(&self, enabled: bool) {
        self.debug_text.store(enabled, Ordering::Relaxed);
    }

    pub fn set_collision(&self, enabled: bool) {
        self.collision.store(enabled, Ordering::Relaxed);
    }

    pub fn set_show_debug_panel(&self, enabled: bool) {
        self.show_debug_panel.store(enabled, Ordering::Relaxed);
    }

    pub fn is_debug_text_enabled(&self) -> bool {
        self.debug_text.load(Ordering::Relaxed)
    }

    pub fn is_collision_enabled(&self) -> bool {
        self.collision.load(Ordering::Relaxed)
    }

    pub fn is_debug_panel_visible(&self) -> bool {
        self.show_debug_panel.load(Ordering::Relaxed)
    }
}

static DEBUG_FLAGS: DebugFlags = DebugFlags {
    debug_text: AtomicBool::new(false),
    collision: AtomicBool::new(false),
    show_debug_panel: AtomicBool::new(false),
};

pub fn debug_flags() -> &'static DebugFlags {
    &DEBUG_FLAGS
}

/// Simple debug overlay using sokol debugtext
pub struct DebugOverlay {
    frame_times: Vec<f32>,
    max_frame_samples: usize,
}

impl DebugOverlay {
    pub fn new() -> Self {
        // Initialize sokol debugtext for simple overlay text
        let mut desc = sdtx::Desc::new();
        desc.fonts[0] = sdtx::font_kc853(); // Simple builtin font
        sdtx::setup(&desc);

        Self {
            frame_times: Vec::new(),
            max_frame_samples: 60,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Just track frame times
        let frame_time = delta_time * 1000.0;
        self.frame_times.push(frame_time);
        if self.frame_times.len() > self.max_frame_samples {
            self.frame_times.remove(0);
        }
    }

    pub fn update_frame_stats(&mut self, dt: f32) {
        // Compatibility method - does same as update
        self.update(dt);
    }

    pub fn render(&mut self) {
        if !debug_flags().is_debug_panel_visible() {
            return;
        }

        // Use sokol debugtext for simple overlay
        sdtx::canvas(sapp::widthf(), sapp::heightf());
        sdtx::origin(10.0, 10.0); // 10px from top-left
        sdtx::home();
        sdtx::color3b(255, 255, 255);

        if !self.frame_times.is_empty() {
            let avg_frame_time =
                self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            let fps = 1000.0 / avg_frame_time.max(0.001);

            sdtx::puts("=== DEBUG PANEL ===\n");
            sdtx::puts(&format!("FPS: {:.1}\n", fps));
            sdtx::puts(&format!("Frame: {:.2}ms\n", avg_frame_time));
            sdtx::puts(&format!("Window: {}x{}\n", sapp::width(), sapp::height()));
            sdtx::puts("\n");

            if debug_flags().is_debug_text_enabled() {
                sdtx::puts("Debug Text: ON\n");
            }
            if debug_flags().is_collision_enabled() {
                sdtx::puts("Collision Debug: ON\n");
            }

            sdtx::puts("\nHotkeys:\n");
            sdtx::puts("F1: Toggle Debug Text\n");
            sdtx::puts("F2: Toggle Collision\n");
            sdtx::puts("F3: Toggle This Panel\n");
        }

        sdtx::draw();
    }

    // Remove this method - no longer needed
    pub fn handle_event(&mut self, _event: &sapp::Event) -> bool {
        false // Never consume events
    }
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if $crate::engine::debug::debug_flags().is_debug_text_enabled() {
            println!("[DEBUG {}:{}] {}", file!(), line!(), format!($($arg)*));
        }
    };
}

/// Toggle debug text
pub fn toggle_debug_text() {
    let current = DEBUG_FLAGS.is_debug_text_enabled();
    DEBUG_FLAGS.set_debug_text(!current);
    println!("Debug text: {}", if !current { "ON" } else { "OFF" });
}

/// Toggle collision debug
pub fn toggle_collision_debug() {
    let current = DEBUG_FLAGS.is_collision_enabled();
    DEBUG_FLAGS.set_collision(!current);
    println!("Collision debug: {}", if !current { "ON" } else { "OFF" });
}

pub fn toggle_debug_panel() {
    let current = DEBUG_FLAGS.is_debug_panel_visible();
    DEBUG_FLAGS.set_show_debug_panel(!current);
    println!("Debug panel: {}", if !current { "ON" } else { "OFF" });
}

/// Set debug text flag
pub fn set_debug_text(enabled: bool) {
    DEBUG_FLAGS.set_debug_text(enabled);
}

/// Set collision debug flag
pub fn set_collision_debug(enabled: bool) {
    DEBUG_FLAGS.set_collision(enabled);
}

/// Set debug panel visibility
pub fn set_debug_panel_visible(enabled: bool) {
    DEBUG_FLAGS.set_show_debug_panel(enabled);
}
