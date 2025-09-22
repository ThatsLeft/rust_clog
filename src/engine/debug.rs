// src/engine/debug.rs

use crate::engine::egui_renderer::EguiRenderer;
use glam::Vec2;
use sokol::app as sapp;
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

/// Debug overlay system with egui integration
pub struct DebugOverlay {
    egui_ctx: egui::Context,
    egui_renderer: EguiRenderer,
    last_frame_time: std::time::Instant,
    frame_times: Vec<f32>,
    max_frame_samples: usize,
    shapes: Vec<egui::epaint::ClippedPrimitive>,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            egui_ctx: egui::Context::default(),
            egui_renderer: EguiRenderer::new(),
            last_frame_time: std::time::Instant::now(),
            frame_times: Vec::new(),
            max_frame_samples: 60,
            shapes: Vec::new(),
        }
    }
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            egui_ctx: egui::Context::default(),
            egui_renderer: EguiRenderer::new(),
            last_frame_time: std::time::Instant::now(),
            frame_times: Vec::new(),
            max_frame_samples: 60,
            shapes: Vec::new(),
        }
    }

    /// Handle input events and return true if egui consumed the event
    pub fn handle_event(&mut self, event: &sapp::Event) -> bool {
        let mut raw_input = egui::RawInput::default();

        // Set screen size and DPI
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(sapp::width() as f32, sapp::height() as f32),
        ));
        if let Some(viewport) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            viewport.native_pixels_per_point = Some(sapp::dpi_scale());
        }

        // Calculate delta time
        static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let start_time = *START_TIME.get_or_init(|| std::time::Instant::now());

        let now = std::time::Instant::now();
        raw_input.time = Some(now.duration_since(start_time).as_secs_f64());

        // Convert Sokol events to egui events
        match event._type {
            sapp::EventType::MouseMove => {
                raw_input.events.push(egui::Event::PointerMoved(egui::pos2(
                    event.mouse_x,
                    event.mouse_y,
                )));
            }
            sapp::EventType::MouseDown => {
                let button = match event.mouse_button {
                    sapp::Mousebutton::Left => egui::PointerButton::Primary,
                    sapp::Mousebutton::Right => egui::PointerButton::Secondary,
                    sapp::Mousebutton::Middle => egui::PointerButton::Middle,
                    _ => return false,
                };
                raw_input.events.push(egui::Event::PointerButton {
                    pos: egui::pos2(event.mouse_x, event.mouse_y),
                    button,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            sapp::EventType::MouseUp => {
                let button = match event.mouse_button {
                    sapp::Mousebutton::Left => egui::PointerButton::Primary,
                    sapp::Mousebutton::Right => egui::PointerButton::Secondary,
                    sapp::Mousebutton::Middle => egui::PointerButton::Middle,
                    _ => return false,
                };
                raw_input.events.push(egui::Event::PointerButton {
                    pos: egui::pos2(event.mouse_x, event.mouse_y),
                    button,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            sapp::EventType::KeyDown => {
                if let Some(key) = sokol_key_to_egui_key(event.key_code) {
                    raw_input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
            sapp::EventType::KeyUp => {
                if let Some(key) = sokol_key_to_egui_key(event.key_code) {
                    raw_input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
            _ => return false,
        }

        let output = self.egui_ctx.run(raw_input, |ctx| {
            self.render_debug_ui(ctx);
        });

        self.shapes = self
            .egui_ctx
            .tessellate(output.shapes, output.pixels_per_point);

        // Update renderer with texture changes
        self.egui_renderer.update_textures(&output.textures_delta);

        self.last_frame_time = now;

        // Return true if egui wants to consume this event
        !output.platform_output.events.is_empty()
    }

    /// Update frame time statistics
    pub fn update_frame_stats(&mut self, dt: f32) {
        self.frame_times.push(dt * 1000.0); // Convert to milliseconds
        if self.frame_times.len() > self.max_frame_samples {
            self.frame_times.remove(0);
        }
    }

    /// Render the debug UI
    fn render_debug_ui(&self, ctx: &egui::Context) {
        if !debug_flags().is_debug_panel_visible() {
            return;
        }

        egui::Window::new("🐛 Debug Panel")
            .resizable(true)
            .default_width(300.0)
            .default_height(250.0)
            .show(ctx, |ui| {
                ui.heading("Debug Options");
                ui.separator();

                // Debug flags section
                ui.label("Toggle debug features:");
                ui.add_space(5.0);

                let mut debug_text_enabled = DEBUG_FLAGS.is_debug_text_enabled();
                if ui
                    .checkbox(&mut debug_text_enabled, "📝 Debug Text Output")
                    .changed()
                {
                    DEBUG_FLAGS.set_debug_text(debug_text_enabled);
                }

                let mut collision_enabled = DEBUG_FLAGS.is_collision_enabled();
                if ui
                    .checkbox(&mut collision_enabled, "🔲 Collision Visualization")
                    .changed()
                {
                    DEBUG_FLAGS.set_collision(collision_enabled);
                }

                ui.separator();
                ui.add_space(5.0);

                // Hotkeys info
                ui.horizontal(|ui| {
                    ui.label("Hotkeys:");
                    ui.weak("F1: Text | F2: Collision | F3: Panel");
                });

                ui.separator();
                ui.add_space(5.0);

                // Performance info
                ui.collapsing("📊 Performance", |ui| {
                    if !self.frame_times.is_empty() {
                        let avg_frame_time =
                            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
                        let fps = 1000.0 / avg_frame_time.max(0.001);

                        ui.label(format!("FPS: {:.1}", fps));
                        ui.label(format!("Frame time: {:.2}ms", avg_frame_time));

                        let max_frame_time =
                            self.frame_times.iter().copied().fold(0.0f32, f32::max);
                        let min_frame_time =
                            self.frame_times.iter().copied().fold(1000.0f32, f32::min);

                        ui.label(format!("Max: {:.2}ms", max_frame_time));
                        ui.label(format!("Min: {:.2}ms", min_frame_time));
                    } else {
                        ui.label("Collecting data...");
                    }
                });

                ui.separator();
                ui.add_space(5.0);

                // System info
                ui.collapsing("🖥️ System", |ui| {
                    ui.label(format!("Window: {}x{}", sapp::width(), sapp::height()));
                    ui.label(format!("DPI Scale: {:.1}", sapp::dpi_scale()));
                    ui.label(format!("High DPI: {}", sapp::high_dpi()));
                });
            });
    }

    /// Render the debug overlay
    pub fn render(&mut self) {
        if !debug_flags().is_debug_panel_visible() || self.shapes.is_empty() {
            return;
        }
        let screen_size = Vec2::new(sapp::width() as f32, sapp::height() as f32);
        self.egui_renderer.render(&self.shapes, screen_size); // Use the stored shapes
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

fn sokol_key_to_egui_key(key: sapp::Keycode) -> Option<egui::Key> {
    use egui::Key;
    use sapp::Keycode;

    match key {
        Keycode::Space => Some(Key::Space),
        Keycode::Comma => Some(Key::Comma),
        Keycode::Minus => Some(Key::Minus),
        Keycode::Period => Some(Key::Period),
        Keycode::Slash => Some(Key::Slash),
        Keycode::Num0 => Some(Key::Num0),
        Keycode::Num1 => Some(Key::Num1),
        Keycode::Num2 => Some(Key::Num2),
        Keycode::Num3 => Some(Key::Num3),
        Keycode::Num4 => Some(Key::Num4),
        Keycode::Num5 => Some(Key::Num5),
        Keycode::Num6 => Some(Key::Num6),
        Keycode::Num7 => Some(Key::Num7),
        Keycode::Num8 => Some(Key::Num8),
        Keycode::Num9 => Some(Key::Num9),
        Keycode::Semicolon => Some(Key::Semicolon),
        Keycode::Equal => Some(Key::Equals),
        Keycode::A => Some(Key::A),
        Keycode::B => Some(Key::B),
        Keycode::C => Some(Key::C),
        Keycode::D => Some(Key::D),
        Keycode::E => Some(Key::E),
        Keycode::F => Some(Key::F),
        Keycode::G => Some(Key::G),
        Keycode::H => Some(Key::H),
        Keycode::I => Some(Key::I),
        Keycode::J => Some(Key::J),
        Keycode::K => Some(Key::K),
        Keycode::L => Some(Key::L),
        Keycode::M => Some(Key::M),
        Keycode::N => Some(Key::N),
        Keycode::O => Some(Key::O),
        Keycode::P => Some(Key::P),
        Keycode::Q => Some(Key::Q),
        Keycode::R => Some(Key::R),
        Keycode::S => Some(Key::S),
        Keycode::T => Some(Key::T),
        Keycode::U => Some(Key::U),
        Keycode::V => Some(Key::V),
        Keycode::W => Some(Key::W),
        Keycode::X => Some(Key::X),
        Keycode::Y => Some(Key::Y),
        Keycode::Z => Some(Key::Z),
        Keycode::LeftBracket => Some(Key::OpenBracket),
        Keycode::Backslash => Some(Key::Backslash),
        Keycode::RightBracket => Some(Key::CloseBracket),
        Keycode::Escape => Some(Key::Escape),
        Keycode::Enter => Some(Key::Enter),
        Keycode::Tab => Some(Key::Tab),
        Keycode::Backspace => Some(Key::Backspace),
        Keycode::Insert => Some(Key::Insert),
        Keycode::Delete => Some(Key::Delete),
        Keycode::Right => Some(Key::ArrowRight),
        Keycode::Left => Some(Key::ArrowLeft),
        Keycode::Down => Some(Key::ArrowDown),
        Keycode::Up => Some(Key::ArrowUp),
        Keycode::PageUp => Some(Key::PageUp),
        Keycode::PageDown => Some(Key::PageDown),
        Keycode::Home => Some(Key::Home),
        Keycode::End => Some(Key::End),
        Keycode::F1 => Some(Key::F1),
        Keycode::F2 => Some(Key::F2),
        Keycode::F3 => Some(Key::F3),
        Keycode::F4 => Some(Key::F4),
        Keycode::F5 => Some(Key::F5),
        Keycode::F6 => Some(Key::F6),
        Keycode::F7 => Some(Key::F7),
        Keycode::F8 => Some(Key::F8),
        Keycode::F9 => Some(Key::F9),
        Keycode::F10 => Some(Key::F10),
        Keycode::F11 => Some(Key::F11),
        Keycode::F12 => Some(Key::F12),
        _ => None,
    }
}
