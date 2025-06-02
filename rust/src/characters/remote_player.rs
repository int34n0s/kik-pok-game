use crate::handle_player_animation;

use godot::prelude::*;

use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};

const MIN_DISTANCE_TO_SNAP: f32 = 0.3;
const INTERPOLATION_LERP_WEIGHT: f32 = 9.0;

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

    fn physics_process(&mut self, delta: f64) {
        let current_global_pos = self.base().get_global_position();
        let target_global_pos = self.position;

        let distance_to_target = current_global_pos.distance_to(target_global_pos);

        let direction = self.position.x - current_global_pos.x;
        let is_on_floor = self.base().is_on_floor();
        if let Some(animated_sprite) = &mut self.animated_sprite {
            handle_player_animation(animated_sprite, direction, is_on_floor);
        }

        self.base_mut().move_and_slide();

        if distance_to_target > MIN_DISTANCE_TO_SNAP {
            let interpolation_alpha = (INTERPOLATION_LERP_WEIGHT * delta as f32).clamp(0.0, 1.0);

            let new_interpolated_pos =
                current_global_pos.lerp(target_global_pos, interpolation_alpha);

            self.base_mut().set_global_position(new_interpolated_pos);
        } else {
            self.position = target_global_pos;

            self.base_mut().set_global_position(target_global_pos);
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
