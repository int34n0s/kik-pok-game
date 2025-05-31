use godot::classes::{CharacterBody2D, ICharacterBody2D, Input, AnimatedSprite2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    speed: f32,
    jump_velocity: f32,

    is_local_player: bool,
    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        godot_print!("Player initialized in Rust!");

        Self {
            speed: 200.0,
            jump_velocity: -300.0,
            is_local_player: false,
            animated_sprite: None,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.is_local_player {
            self.handle_local_input(delta);
        }
    }

    fn ready(&mut self) {
        godot_print!("Player ready!");

        if let Some(first_child) = self.base().get_child(0) {
            if let Ok(animated_sprite) = first_child.try_cast::<AnimatedSprite2D>() {
                self.animated_sprite = Some(animated_sprite);
            }
        }
    }
}

#[godot_api]
impl Player {
    /// Mark this player as the local controllable player
    #[func]
    pub fn set_as_local_player(&mut self) {
        self.is_local_player = true;
        godot_print!("Player set as local player");
    }

    /// Mark this player as a remote player (other players)
    #[func]
    pub fn set_as_remote_player(&mut self) {
        self.is_local_player = false;
        godot_print!("Player set as remote player");
    }

    /// Set the player's position (used for remote players)
    #[func]
    pub fn set_player_position(&mut self, position: Vector2) {
        self.base_mut().set_global_position(position);
    }

    /// Get the player's position (used for local player position sending)
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
            // Flip the sprite
            if direction > 0.0 {
                animated_sprite.set_flip_h(false);
            } else if direction < 0.0 {
                animated_sprite.set_flip_h(true);
            }
            
            // Play animations
            if is_on_floor {
                if direction == 0.0 {
                    animated_sprite.call("play",&["idle".to_variant()]);
                } else {
                    animated_sprite.call("play",&["run".to_variant()]);
                }
            } else {
                animated_sprite.call("play",&["jump".to_variant()]);
            }
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
    }
}
