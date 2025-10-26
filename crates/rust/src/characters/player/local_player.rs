use super::BasicPlayer;
use std::sync::{Arc, Mutex};

use crate::{
    DbConnection, DbPlayerState, DbVector2, RegistrationState, SpacetimeDBManager, register_player,
};

use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input};
use godot::prelude::*;

use spacetimedb_sdk::DbContext;

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
    pub fn setup_multiplayer(
        connection: &DbConnection,
        registration_state: Arc<Mutex<RegistrationState>>,
    ) {
        connection
            .subscription_builder()
            .subscribe("SELECT * FROM player");

        let registration_state = registration_state.clone();
        connection
            .reducers
            .on_register_player(move |ctx, _name, _scene_id| match &ctx.event.status {
                spacetimedb_sdk::Status::Committed => {
                    godot_print!("Player registration committed successfully");

                    let mut state = registration_state.lock().unwrap();
                    *state = RegistrationState::Registered;
                }
                spacetimedb_sdk::Status::Failed(e) => {
                    godot_print!("Player registration failed: {}", e);

                    let mut state = registration_state.lock().unwrap();
                    *state = RegistrationState::RegistrationFailed(e.to_string());
                }
                spacetimedb_sdk::Status::OutOfEnergy => {
                    godot_print!("Player registration failed: Out of energy");

                    let mut state = registration_state.lock().unwrap();
                    *state = RegistrationState::RegistrationFailed("Out of energy".to_string());
                }
            });
    }

    #[func]
    pub fn get_player_position(&self) -> Vector2 {
        self.base().get_global_position()
    }

    fn handle_local_input(&mut self, delta: f64) {
        let input = Input::singleton();
        let mut velocity = self.base().get_velocity();
        let is_on_floor = self.base().is_on_floor();

        let base = self.base();

        let jump_pressed = input.is_action_just_pressed("jump");
        let direction = input.get_axis("move_left", "move_right");

        // Apply gravity
        if !is_on_floor {
            velocity.y += base.get_gravity().y * delta as f32;
        }

        godot_print!("Velocity before jump: {:?}", velocity);

        velocity = self.basic_player.handle_jump(
            velocity,
            self.base().get_platform_velocity(),
            is_on_floor,
            jump_pressed,
        );

        godot_print!("Velocity after jump: {:?}", velocity);

        // Apply horizontal movement using basic player
        self.basic_player
            .apply_horizontal_movement(&mut velocity, direction, is_on_floor);

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

        let state = DbPlayerState {
            position: DbVector2::from(self.base().get_position()),
            direction: direction as i32,
            is_jumping,
        };

        let Some(connection) = SpacetimeDBManager::get_read_connection() else {
            godot_print!("Could not get database connection!");
            return;
        };

        match connection.send_inputs(state) {
            Ok(_) => {}
            Err(e) => godot_print!("Failed to send inputs: {}", e),
        }
    }
}
