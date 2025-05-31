use godot::prelude::*;

use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input};

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

#[derive(GodotClass)]
#[class(init)]
pub struct PlayerState {
    #[export]
    pub position: Vector2,
    #[export]
    pub direction: f32,
    #[export]
    pub is_on_floor: bool,
}

#[godot_api]
impl PlayerState {
    #[func]
    pub fn new(position: Vector2, direction: f32, is_on_floor: bool) -> Gd<Self> {
        Gd::from_object(Self {
            position,
            direction,
            is_on_floor,
        })
    }
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
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
        if let Some(animated_sprite) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            self.animated_sprite = Some(animated_sprite);
            return;
        }

        if self.animated_sprite.is_none() {
            godot_print!("Failed to find animated sprite");
        }
    }
}

#[godot_api]
impl Player {
    #[func]
    pub fn set_as_local_player(&mut self) {
        self.is_local_player = true;
    }

    #[func]
    pub fn set_as_remote_player(&mut self) {
        self.is_local_player = false;
    }

    #[func]
    pub fn is_player_local(&self) -> bool {
        self.is_local_player
    }

    #[func]
    pub fn set_player_state(&mut self, state: Gd<PlayerState>) {
        let state = state.bind();

        let position = state.position;
        let direction = state.direction;
        let is_on_floor = state.is_on_floor;

        self.base_mut().set_global_position(position);

        if let Some(animated_sprite) = &mut self.animated_sprite {
            Self::handle_animated_sprite(animated_sprite, direction, is_on_floor);
        }
    }

    #[func]
    pub fn get_player_state(&self) -> Gd<PlayerState> {
        let position = self.base().get_global_position();
        let direction = Input::singleton().get_axis("move_left", "move_right");
        let is_on_floor = self.base().is_on_floor();

        PlayerState::new(position, direction, is_on_floor)
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
            Self::handle_animated_sprite(animated_sprite, direction, is_on_floor);
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

    fn handle_animated_sprite(
        animated_sprite: &mut AnimatedSprite2D,
        direction: f32,
        is_on_floor: bool,
    ) {
        // Flip the sprite
        if direction > 0.0 {
            animated_sprite.set_flip_h(false);
        } else if direction < 0.0 {
            animated_sprite.set_flip_h(true);
        }

        // Play animations
        if is_on_floor {
            if direction == 0.0 {
                animated_sprite.call("play", &["idle".to_variant()]);
            } else {
                animated_sprite.call("play", &["run".to_variant()]);
            }
        } else {
            animated_sprite.call("play", &["jump".to_variant()]);
        }
    }
}
