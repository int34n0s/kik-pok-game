use crate::*;

use godot::classes::{INode, Label, Node, PackedScene, ResourceLoader};
use godot::prelude::*;

use spacetimedb_sdk::Identity;
use std::collections::HashMap;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MultiplayerManager {
    #[allow(dead_code)]
    position_update_timer: f32,
    #[allow(dead_code)]
    position_update_interval: f32,

    /// player_id -> Player instance
    remote_players: HashMap<Identity, Gd<RemotePlayerNode>>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for MultiplayerManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            position_update_timer: 0.0,
            position_update_interval: 1.0 / 60.0, // 60 FPS position updates
            remote_players: HashMap::new(),
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        self.handle_multiplayer_updates();
    }

    fn ready(&mut self) {}
}

#[godot_api]
impl MultiplayerManager {
    fn handle_multiplayer_updates(&mut self) {
        {
            let db_manager = GLOBAL_CONNECTION.lock().unwrap();
            if let Err(e) = db_manager.tick() {
                godot_print!("Database tick error: {}", e);
            }
        }

        self.sync_remote_players();
    }

    fn sync_remote_players(&mut self) {
        let db_manager = GLOBAL_CONNECTION.lock().unwrap();

        let players = db_manager.get_other_players();

        // Track active remote players
        let mut current_remote_players = std::collections::HashSet::new();

        // Process all entities (except our own)
        for player in players {
            current_remote_players.insert(player.identity);

            if let Some(remote_player) = self.remote_players.get_mut(&player.identity) {
                remote_player
                    .bind_mut()
                    .set_player_position(player.positioning.into());

                continue;
            }

            self.spawn_remote_player(&player);
        }

        // Remove remote players who left
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
        let position = Vector2::new(player.positioning.x, player.positioning.y);

        godot_print!(
            "Spawning remote player {} ({}) at ({}, {})",
            player_id,
            name,
            position.x,
            position.y
        );

        // Load the player scene
        let player_scene_path = "res://scenes/characters/remote_player.tscn";
        let mut resource_loader = ResourceLoader::singleton();

        let Some(packed_scene) = resource_loader.load(player_scene_path) else {
            godot_print!("Failed to load resource at {}", player_scene_path);
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

        // Configure the remote player
        remote_player.set_position(position);
        remote_player.set_name(&GString::from(name));

        if let Some(mut player_name) = remote_player.try_get_node_as::<Label>("PlayerName") {
            player_name.set_text(&GString::from(name));
        }

        // Add to scene
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
