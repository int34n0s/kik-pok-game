use crate::*;

use godot::classes::{AnimationPlayer, Area2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct CoinArea {
    game_manager: Option<Gd<GameManager>>,
    animation_player: Option<Gd<AnimationPlayer>>,

    #[base]
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for CoinArea {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            game_manager: None,
            animation_player: None,
            base,
        }
    }

    fn ready(&mut self) {
        if let Some(mut tree) = self.base().get_tree() {
            if let Some(manager_node) = tree.get_first_node_in_group("manager") {
                self.game_manager = manager_node.try_cast::<GameManager>().ok();
            }
        }

        if self.game_manager.is_none() {
            godot_error!("Could not find GameManager node in 'manager' group");
        }

        self.animation_player = self
            .base()
            .try_get_node_as::<AnimationPlayer>("AnimationPlayer");

        if self.animation_player.is_none() {
            godot_error!("Could not find AnimationPlayer node");
        }

        let position = self.base().get_global_position();
        let scene_id = 1; // TODO: Default scene ID - you might want to get this dynamically

        {
            let connection = GLOBAL_CONNECTION.lock().unwrap();
            if connection.is_connected() {
                if let Err(e) = connection.register_coin_at_position(position, scene_id) {
                    godot_warn!("Failed to register coin position: {}", e);
                }
            }
        }

        self.check_if_collected_and_remove();

        let callable = self.base().callable("on_body_entered");
        self.base_mut().connect("body_entered", &callable);
    }
}

#[godot_api]
impl CoinArea {
    #[func]
    fn on_body_entered(&mut self, body: Gd<PlayerNode>) {
        let position = self.base().get_global_position();

        {
            let connection = GLOBAL_CONNECTION.lock().unwrap();
            if !connection.is_connected() {
                return;
            }

            match connection.collect_coin_at_position(position) {
                Ok(_) => {
                    godot_print!(
                        "Coin collected successfully at ({}, {})!",
                        position.x,
                        position.y
                    );
                }
                Err(e) => {
                    godot_error!("Failed to collect coin: {}", e);
                    return;
                }
            }
        }

        let connection = GLOBAL_CONNECTION.lock().unwrap();
        if !connection.is_connected() {
            return;
        }

        let body_binding = body.bind();
        if !body_binding.is_player_local() {
            self.base_mut().queue_free();

            return;
        }

        // Play pickup animation
        if let Some(animation_player) = &mut self.animation_player {
            animation_player.play_ex().name("pickup").done();
        } else {
            godot_error!("AnimationPlayer not available to play pickup animation");
        }
    }

    fn check_if_collected_and_remove(&mut self) {
        let position = self.base().get_global_position();

        let connection = GLOBAL_CONNECTION.lock().unwrap();
        if connection.is_coin_collected_at_position(position) {
            godot_print!(
                "Coin at ({}, {}) was already collected, removing from scene",
                position.x,
                position.y
            );
            self.base_mut().queue_free();
        }
    }
}
