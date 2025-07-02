use crate::multiplayer::bootstrap::WorldBootstrap;
use crate::*;

use godot::classes::{Engine, INode, Label, Node, PackedScene, ResourceLoader};
use godot::prelude::*;

use spacetimedb_sdk::Identity;
use std::collections::HashMap;

pub const FRAME_RATE: f32 = 60.0;

pub const PLAYER_SCENE_PATH: &str = "res://scenes/characters/remote_player.tscn";

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

    fn process(&mut self, _delta: f64) {
        self.handle_multiplayer_updates();
    }

    fn ready(&mut self) {
        Engine::singleton().set_max_fps(FRAME_RATE as i32);

        let bootstrap = WorldBootstrap::new();

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

        let Err(result) = bootstrap.start(&mut self.base_mut(), connection, player_name) else {
            return;
        };

        godot_print!("Failed to start bootstrap: {:?}", result);
    }
}

#[godot_api]
impl MultiplayerManager {
    fn handle_multiplayer_updates(&mut self) {
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

            self.spawn_remote_player(&player);
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

    fn spawn_remote_player(&mut self, player: &DbPlayer) {
        let player_id = player.identity;
        let name = &player.name;
        let position = Vector2::new(player.state.position.x, player.state.position.y);

        godot_print!(
            "Spawning remote player {} ({}) at ({}, {})",
            player_id,
            name,
            position.x,
            position.y
        );

        let mut resource_loader = ResourceLoader::singleton();
        let Some(packed_scene) = resource_loader.load(PLAYER_SCENE_PATH) else {
            godot_print!("Failed to load resource at {}", PLAYER_SCENE_PATH);
            return;
        };

        let Ok(scene) = packed_scene.try_cast::<PackedScene>() else {
            godot_print!("Failed to cast resource to PackedScene");
            return;
        };

        let Some(instance) = scene.instantiate() else {
            godot_print!("Failed to instantiate scene");
            return;
        };

        let Ok(mut remote_player) = instance.try_cast::<RemotePlayerNode>() else {
            godot_print!("Failed to cast instance to Player");
            return;
        };

        remote_player.set_position(position);
        remote_player.set_name(&GString::from(player_id.to_string()));

        if let Some(mut player_name) = remote_player.try_get_node_as::<Label>("PlayerName") {
            player_name.set_text(&GString::from(name));
        }

        self.base_mut().add_child(&remote_player);
        self.remote_players.insert(player_id, remote_player);

        godot_print!("Remote player {} spawned successfully", player_id);
    }

    fn remove_remote_player(&mut self, player_id: Identity) {
        if let Some(mut remote_player) = self.remote_players.remove(&player_id) {
            remote_player.queue_free();
            godot_print!("Removed remote player {}", player_id);
        }
    }
}
