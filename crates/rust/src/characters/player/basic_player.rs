use godot::classes::AnimatedSprite2D;

use godot::prelude::*;

use crate::handle_player_animation;

const MOMENTUM_FACTOR: f32 = 0.9;

pub struct BasicPlayer {
    pub speed: f32,
    pub jump_velocity: f32,
    pub animated_sprite: Option<Gd<AnimatedSprite2D>>,

    velocity_in_jump: Option<Vector2>,
}

impl Default for BasicPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicPlayer {
    pub fn new() -> Self {
        Self {
            speed: 100.0,
            jump_velocity: -300.0,
            animated_sprite: None,
            velocity_in_jump: None,
        }
    }

    pub fn handle_jump(
        &mut self,
        mut velocity: Vector2,
        platform_velocity: Vector2,
        is_on_floor: bool,
        jump_pressed: bool,
    ) -> Vector2 {
        // Handle jump using basic player
        if jump_pressed && is_on_floor {
            velocity.y = self.jump_velocity;
            velocity.x += platform_velocity.x;

            self.velocity_in_jump = Some(velocity);
        }

        velocity
    }

    pub fn apply_horizontal_movement(
        &mut self,
        velocity: &mut Vector2,
        direction: f32,
        is_on_floor: bool,
    ) {
        if is_on_floor && velocity.y >= 0.0 {
            self.velocity_in_jump = None;
        }

        if let Some(velocity_in_jump) = self.velocity_in_jump {
            let target = if direction != 0.0 {
                direction * self.speed
            } else {
                velocity_in_jump.x * MOMENTUM_FACTOR
            };
            velocity.x = target;

            return;
        }

        if direction != 0.0 {
            velocity.x = direction * self.speed;
        } else {
            velocity.x =
                godot::global::move_toward(velocity.x as f64, 0.0, self.speed as f64) as f32;
        }
    }

    pub fn handle_animation(&mut self, direction: f32, is_on_floor: bool) {
        if let Some(animated_sprite) = &mut self.animated_sprite {
            handle_player_animation(animated_sprite, direction, is_on_floor);
        }
    }
}
