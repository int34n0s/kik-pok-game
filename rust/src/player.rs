use crate::spacetimedb_manager::SpacetimeDBManager;
use godot::classes::{CharacterBody2D, ICharacterBody2D, Input};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    speed: f32,
    jump_velocity: f32,
    spacetimedb_manager: Option<SpacetimeDBManager>,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        godot_print!("Player initialized in Rust!");

        // Connect to SpacetimeDB
        let spacetimedb_manager = match SpacetimeDBManager::connect("127.0.0.1:3000", "kik-pok")
        {
            Ok(manager) => {
                // Call debug reducer right after initialization
                godot_print!("1");
                
                if let Err(e) = manager.call_debug() {
                    godot_print!("Failed to call debug reducer during init: {}", e);
                }
                
                godot_print!("2");
                
                if let Err(e) = manager.call_debug() {
                    godot_print!("Failed to call debug reducer during init: {}", e);
                }
                
                godot_print!("3");
                
                if let Err(e) = manager.call_debug() {
                    godot_print!("Failed to call debug reducer during init: {}", e);
                }

                Some(manager)
            }
            Err(e) => {
                godot_print!("Failed to connect to SpacetimeDB during player init: {}", e);

                None
            }
        };

        Self {
            speed: 200.0,
            jump_velocity: -280.0,
            spacetimedb_manager,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let input = Input::singleton();
        let mut velocity = self.base().get_velocity();

        // Add gravity when not on floor
        if !self.base().is_on_floor() {
            velocity += self.base().get_gravity() * delta as f32;
        }

        // Handle jump
        if input.is_action_just_pressed("ui_accept") && self.base().is_on_floor() {
            if let Some(manager) = &self.spacetimedb_manager {
                if let Err(e) = manager.call_debug() {
                    godot_print!("Failed to call debug reducer during jump: {}", e);
                }
            } else {
                 godot_print!("MANAGERRRRRR");
            }

            velocity.y = self.jump_velocity;
        }

        // Get input direction and handle movement/deceleration
        let direction = input.get_axis("ui_left", "ui_right");

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
