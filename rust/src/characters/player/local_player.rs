use godot::prelude::*;

use super::BasicPlayer;
use crate::GLOBAL_CONNECTION;
use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct LocalPlayerNode {
    basic_player: BasicPlayer,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for LocalPlayerNode {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            basic_player: BasicPlayer::new(),
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
            self.basic_player.animated_sprite = Some(animated_sprite);
        } else {
            godot_print!("Failed to find animated sprite");
        }
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

        let jump_pressed = input.is_action_just_pressed("jump");
        let direction = input.get_axis("move_left", "move_right");

        // Apply gravity
        if !is_on_floor {
            velocity.y += self.base().get_gravity().y * delta as f32;
        }

        // Handle jump using basic player
        if jump_pressed && is_on_floor {
            self.basic_player.handle_jump(&mut velocity);
        }

        // Apply horizontal movement using basic player
        self.basic_player
            .apply_horizontal_movement(&mut velocity, direction);

        // Update velocity and move
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        // Handle animations using basic player
        self.basic_player.handle_animation(direction, is_on_floor);

        self.send_inputs(direction, jump_pressed, is_on_floor);
    }

    fn send_inputs(&self, direction: f32, jump_pressed: bool, is_on_floor: bool) {
        let updated_velocity = self.base().get_velocity();
        let is_jumping = jump_pressed || (!is_on_floor && updated_velocity.y < 0.0);

        let connection = GLOBAL_CONNECTION.lock().unwrap();
        if !connection.is_connected() {
            return;
        }

        match connection.send_inputs(direction as i32, is_jumping, self.base().get_position()) {
            Ok(_) => {}
            Err(e) => godot_print!("Failed to send inputs: {}", e),
        }
    }
}
