use godot::prelude::*;

use crate::{GLOBAL_CONNECTION, handle_player_animation};
use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct LocalPlayerNode {
    speed: f32,
    jump_velocity: f32,

    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for LocalPlayerNode {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            speed: 200.0,
            jump_velocity: -300.0,
            animated_sprite: None,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        self.handle_local_input(delta);
    }

    fn ready(&mut self) {
        if let Some(animated_sprite) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            self.animated_sprite = Some(animated_sprite);

            return;
        }

        godot_print!("Failed to find animated sprite");
    }
}

#[godot_api]
impl LocalPlayerNode {
    #[func]
    pub fn get_player_position(&self) -> Vector2 {
        self.base().get_global_position()
    }

    fn handle_local_input(&mut self, delta: f64) {
        let input = Input::singleton();
        let mut velocity = self.base().get_velocity();

        let is_on_floor = self.base().is_on_floor();

        // Add gravity when not on floor
        if !is_on_floor {
            velocity.y += self.base().get_gravity().y * delta as f32;
        }

        // Handle jump
        if input.is_action_just_pressed("jump") && is_on_floor {
            velocity.y = self.jump_velocity;
        }

        // Get the input direction: -1, 0, 1
        let direction = input.get_axis("move_left", "move_right");

        // Handle sprite flipping and animations
        if let Some(animated_sprite) = &mut self.animated_sprite {
            handle_player_animation(animated_sprite, direction, is_on_floor);
        }

        // Apply movement
        if direction != 0.0 {
            velocity.x = direction * self.speed;
        } else {
            velocity.x =
                godot::global::move_toward(velocity.x as f64, 0.0, self.speed as f64) as f32;
        }

        // Update velocity and move
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        if direction != 0.0 || !is_on_floor {
            let connection = GLOBAL_CONNECTION.lock().unwrap();
            if !connection.is_connected() {
                return;
            }

            let current_position = self.base().get_global_position();
            match connection.update_position(current_position.into()) {
                Ok(_) => {}
                Err(e) => godot_print!("Failed to send position: {}", e),
            }
        }
    }
}
