use crate::multiplayer::bootstrap::WorldBootstrap;
use crate::*;

use godot::classes::{INode, Node};
use godot::prelude::*;

use spacetimedb_sdk::Identity;
use std::collections::HashMap;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MultiplayerManager {
    remote_players: HashMap<Identity, Gd<RemotePlayerNode>>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for MultiplayerManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            remote_players: HashMap::new(),
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        self.handle_multiplayer_updates(delta as f32);
    }

    fn ready(&mut self) {
        let Some(db_manager) = SpacetimeDBManager::get_read_connection() else {
            godot_print!("Failed to get database connection");
            return;
        };

        let Ok(connection) = db_manager.get_connection() else {
            godot_print!("Failed to get connection");
            return;
        };

        let Some(player_name) = db_manager.login_module.get_player_name() else {
            godot_print!("Failed to get player name");
            return;
        };

        let bootstrap = WorldBootstrap::new();
        if let Err(result) = bootstrap.boot_player(&mut self.base_mut(), connection, player_name) {
            godot_print!("Failed to start bootstrap: {:?}", result);
        }
    }
}

#[godot_api]
impl MultiplayerManager {
    fn handle_multiplayer_updates(&mut self, _delta: f32) {
        {
            let Some(mut db_manager) = SpacetimeDBManager::get_write_connection() else {
                godot_print!("Failed to get database connection for tick");
                return;
            };

            if let Err(e) = db_manager.tick() {
                godot_print!("Database tick error: {}", e);
            }
        }

        self.sync_remote_players();
    }

    fn sync_remote_players(&mut self) {
        let Some(db_manager) = SpacetimeDBManager::get_read_connection() else {
            return;
        };

        let Ok(players) = db_manager.get_other_players() else {
            return;
        };

        let mut current_remote_players = std::collections::HashSet::new();
        for player in players {
            current_remote_players.insert(player.identity);

            if let Some(remote_player) = self.remote_players.get_mut(&player.identity) {
                let position = Vector2::new(player.state.position.x, player.state.position.y);

                remote_player.bind_mut().set_player_position(
                    player.state.direction,
                    player.state.is_jumping,
                    position,
                );

                continue;
            }

            let Ok(remote_player) = RemotePlayerNode::spawn_object(self.base_mut(), &player) else {
                godot_print!("Failed to spawn remote player");
                return;
            };

            self.remote_players.insert(player.identity, remote_player);
        }

        let players_to_remove: Vec<Identity> = self
            .remote_players
            .keys()
            .filter(|&player_id| !current_remote_players.contains(player_id))
            .cloned()
            .collect();

        for player_id in players_to_remove {
            self.remove_remote_player(player_id);
        }
    }

    fn remove_remote_player(&mut self, player_id: Identity) {
        if let Some(mut remote_player) = self.remote_players.remove(&player_id) {
            remote_player.queue_free();
            godot_print!("Removed remote player {}", player_id);
        }
    }
}
