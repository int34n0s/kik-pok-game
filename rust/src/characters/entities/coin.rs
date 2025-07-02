use crate::*;

use godot::classes::{AnimationPlayer, Area2D, IArea2D};
use godot::prelude::*;

use spacetimedb_sdk::DbContext;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct CoinNode {
    game_manager: Option<Gd<GameManager>>,

    animation_player: Option<Gd<AnimationPlayer>>,

    #[base]
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for CoinNode {
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

        let callable = self.base().callable("on_body_entered");
        self.base_mut().connect("body_entered", &callable);
    }
}

#[godot_api]
impl CoinNode {
    pub fn setup_multiplayer(connection: &DbConnection) {
        connection
            .subscription_builder()
            .subscribe("SELECT * FROM coin");
    }

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        let position = self.base().get_global_position();

        {
            let Some(connection) = SpacetimeDBManager::get_read_connection() else {
                godot_print!("No connection!");

                return;
            };

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

        let local_player = body.try_cast::<LocalPlayerNode>();
        if local_player.is_err() {
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

    #[func]
    fn remove(&mut self) {
        self.base_mut().queue_free();
    }
}
