use std::collections::HashMap;

use glam::{Vec2, Vec4};

use crate::engine::Sprite;


pub struct SpriteAnimations {
    pub name: String,
    pub texture_name: String,
    pub frame_size: Vec2,
    pub frame_count: u32,
    pub frames_per_row: u32,
    pub duration: f32,
    pub looping: bool,
}

#[derive(Clone, Debug)]
pub struct AnimationState {
    pub current_frame: u32,
    pub elapsed_time: f32,
    pub is_playing: bool,
    pub current_animation: Option<String>,
}

pub struct AnimationManager {
    animations: HashMap<String, SpriteAnimations>
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }

    pub fn update_sprite_animation(&self, sprite: &mut Sprite, dt: f32) {
        if let Some(ref mut anim_state) = sprite.animation_state {
            if !anim_state.is_playing {
                return;
            }

            if let Some(ref anim_name) = anim_state.current_animation {
                if let Some(animation) = self.animations.get(anim_name) {
                    // Update time
                    anim_state.elapsed_time += dt;
                    
                    // Calculate current frame
                    let frame_duration = animation.duration / animation.frame_count as f32;
                    let frame_index = (anim_state.elapsed_time / frame_duration) as u32;
                    
                    if frame_index >= animation.frame_count {
                        if animation.looping {
                            anim_state.elapsed_time = 0.0;
                            anim_state.current_frame = 0;
                        } else {
                            anim_state.current_frame = animation.frame_count - 1;
                            anim_state.is_playing = false;
                        }
                    } else {
                        anim_state.current_frame = frame_index;
                    }
                    
                    // Calculate UV coordinates for current frame
                    let frame_width = animation.frame_size.x;
                    let frame_height = animation.frame_size.y;
                    
                    let col = anim_state.current_frame % animation.frames_per_row;
                    let row = anim_state.current_frame / animation.frames_per_row;
                    
                    // Assume spritesheet dimensions - you'll need actual texture size
                    let sheet_width = animation.frames_per_row as f32 * frame_width;
                    let sheet_height = ((animation.frame_count + animation.frames_per_row - 1) / animation.frames_per_row) as f32 * frame_height;
                    
                    sprite.uv = Vec4::new(
                        col as f32 * frame_width / sheet_width,      // u
                        row as f32 * frame_height / sheet_height,    // v
                        frame_width / sheet_width,                   // width
                        frame_height / sheet_height,                 // height
                    );
                }
            }
        }
    }

    pub fn play_animation(&self, sprite: &mut Sprite, animation_name: &str) {
        sprite.animation_state = Some(AnimationState {
            current_frame: 0,
            elapsed_time: 0.0,
            is_playing: true,
            current_animation: Some(animation_name.to_string()),
        });
    }

    pub fn register_animation(&mut self, animation: SpriteAnimations) {
        self.animations.insert(animation.name.clone(), animation);
    }

    pub fn stop_animation(&self, sprite: &mut Sprite) {
        if let Some(ref mut anim_state) = sprite.animation_state {
            anim_state.is_playing = false;
        }
    }

    pub fn clear_animation(&self, sprite: &mut Sprite) {
        sprite.animation_state = None;
    }
}