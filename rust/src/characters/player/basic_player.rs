use godot::prelude::*;
use godot::classes::{AnimatedSprite2D};

use crate::handle_player_animation;

pub struct BasicPlayer {
    pub speed: f32,
    pub jump_velocity: f32,
    pub animated_sprite: Option<Gd<AnimatedSprite2D>>,
}

impl BasicPlayer {
    pub fn new() -> Self {
        Self {
            speed: 100.0,
            jump_velocity: -300.0,
            animated_sprite: None,
        }
    }

    pub fn handle_jump(&self, velocity: &mut Vector2) {
        velocity.y = self.jump_velocity;
    }

    pub fn apply_horizontal_movement(&self, velocity: &mut Vector2, direction: f32) {
        if direction != 0.0 {
            velocity.x = direction * self.speed;
        } else {
            velocity.x = godot::global::move_toward(velocity.x as f64, 0.0, self.speed as f64) as f32;
        }
    }

    pub fn handle_animation(&mut self, direction: f32, is_on_floor: bool) {
        if let Some(animated_sprite) = &mut self.animated_sprite {
            handle_player_animation(animated_sprite, direction, is_on_floor);
        }
    }
}