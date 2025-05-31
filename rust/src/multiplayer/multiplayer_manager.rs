use crate::player::Player;
use crate::{GLOBAL_CONNECTION, player_type};

use godot::classes::{INode, Label, Node, PackedScene, ResourceLoader};
use godot::prelude::*;

use std::collections::HashMap;

use spacetimedb_sdk::Identity;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MultiplayerManager {
    position_update_timer: f32,
    position_update_interval: f32,

    // Player management
    local_player: Option<Gd<Player>>,
    remote_players: HashMap<Identity, Gd<Player>>, // player_id -> Player instance

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for MultiplayerManager {
    fn init(base: Base<Node>) -> Self {
        godot_print!("MultiplayerManager initialized!");

        Self {
            position_update_timer: 0.0,
            position_update_interval: 1.0 / 60.0, // 60 FPS position updates
            local_player: None,
            remote_players: HashMap::new(),
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        self.handle_multiplayer_updates(delta);
    }

    fn ready(&mut self) {
        godot_print!("MultiplayerManager starting connection...");

        let Some(mut player) = self.base().try_get_node_as::<Player>("Player") else {
            godot_print!("Failed to find player!");
            return;
        };

        {
            let mut binding = player.bind_mut();
            binding.set_as_local_player();
        }

        self.local_player = Some(player);
    }
}

#[godot_api]
impl MultiplayerManager {
    fn handle_multiplayer_updates(&mut self, delta: f64) {
        {
            let db_manager = GLOBAL_CONNECTION.lock().unwrap();

            if let Err(e) = db_manager.tick() {
                godot_print!("Database tick error: {}", e);
            }
        }

        self.position_update_timer += delta as f32;
        if self.position_update_timer >= self.position_update_interval {
            self.send_local_player_position();
            self.position_update_timer = 0.0;
        }

        self.sync_remote_players();
    }

    fn send_local_player_position(&mut self) {
        let Some(local_player) = &mut self.local_player else {
            return;
        };

        let player_state = local_player.bind().get_player_state();
        let state = player_state.bind();

        let db_manager = GLOBAL_CONNECTION.lock().unwrap();

        if !db_manager.is_logged_in() {
            return;
        }

        match db_manager.update_position(state.to_positioning()) {
            Ok(_) => {}
            Err(e) => godot_print!("Failed to send position: {}", e),
        }
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
                let mut binding = remote_player.bind_mut();
                binding.set_player_state(Gd::from_object(player.positioning.to_player_state()));

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

    fn spawn_remote_player(&mut self, player: &player_type::Player) {
        let player_id = player.identity;
        let name = &player.name;
        let position = Vector2::new(
            player.positioning.coordinates.x,
            player.positioning.coordinates.y,
        );

        godot_print!(
            "Spawning remote player {} ({}) at ({}, {})",
            player_id,
            name,
            position.x,
            position.y
        );

        // Load the player scene
        let player_scene_path = "res://scenes/characters/player.tscn";
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

        let Ok(mut remote_player) = instance.try_cast::<Player>() else {
            godot_print!("Failed to cast instance to Player");
            return;
        };

        // Configure the remote player
        remote_player.set_position(position);
        remote_player.set_name(&GString::from(name));

        {
            let mut remote_player_binding = remote_player.bind_mut();
            remote_player_binding.set_as_remote_player();
        }

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
