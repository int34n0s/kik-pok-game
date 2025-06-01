use crate::handle_player_animation;

use godot::prelude::*;

use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct RemotePlayerNode {
    position: Vector2,

    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for RemotePlayerNode {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            position: Vector2::default(),
            animated_sprite: None,
            base,
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.base_mut().move_and_slide();

        let new_position = self.position;
        let current_position = self.base().get_global_position();

        self.base_mut().set_global_position(new_position);

        let direction = self.position.x - current_position.x;
        let is_on_floor = self.base().is_on_floor() && self.position.y == current_position.y;

        if let Some(animated_sprite) = &mut self.animated_sprite {
            handle_player_animation(animated_sprite, direction, is_on_floor);
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
impl RemotePlayerNode {
    #[func]
    pub fn get_player_state(&self) -> Vector2 {
        self.base().get_global_position()
    }

    #[func]
    pub fn set_player_position(&mut self, state: Vector2) {
        self.position = state;
    }
}
