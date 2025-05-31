
use crate::SpacetimeDBManager;
use crate::player::Player;
use crate::PlayerTableAccess;

use fastrand;
use godot::classes::{INode, Label, Node, PackedScene, ResourceLoader};
use godot::prelude::*;
use spacetimedb_sdk::{DbContext, Table};
use std::collections::HashMap;


#[derive(GodotClass)]
#[class(base=Node)]
pub struct MultiplayerManager {
    db_manager: Option<SpacetimeDBManager>,
    position_update_timer: f32,
    position_update_interval: f32,

    // Player management
    local_player: Option<Gd<Player>>,
    local_player_id: Option<u32>,
    remote_players: HashMap<u32, Gd<Player>>, // player_id -> Player instance

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for MultiplayerManager {
    fn init(base: Base<Node>) -> Self {
        godot_print!("MultiplayerManager initialized!");

        Self {
            db_manager: None,
            position_update_timer: 0.0,
            position_update_interval: 1.0 / 60.0, // 10 FPS position updates
            local_player: None,
            local_player_id: None,
            remote_players: HashMap::new(),
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        self.handle_multiplayer_updates(delta);
    }

    fn ready(&mut self) {
        godot_print!("MultiplayerManager starting connection...");
        
        self.connect_to_multiplayer();
    }
}

#[godot_api]
impl MultiplayerManager {
    fn connect_to_multiplayer(&mut self) {
        self.db_manager = Some(SpacetimeDBManager::new());

        if let Some(db_manager) = &mut self.db_manager {
            match db_manager.connect("127.0.0.1:3000", "kik-pok") {
                Ok(_) => {
                    godot_print!("Connected to SpacetimeDB server");

                    // Register this client as a player
                    let player_name = format!("Player_{}", fastrand::u32(1000..9999));
                    match db_manager.register_player(player_name.clone()) {
                        Ok(_) => {
                            godot_print!("Registered as player: {}", player_name);
                            self.spawn_local_player();
                        }
                        Err(e) => {
                            godot_print!("Failed to register player: {}", e);
                        }
                    }
                }
                Err(e) => {
                    godot_print!("Failed to connect to server: {}", e);
                }
            }
        }
    }

    fn spawn_local_player(&mut self) {
        godot_print!("Setting up local player...");
        
        let children_count = self.base().get_child_count();
        if children_count != 1 {
            return;
        }
        
        let child = self.base().get_child(0).unwrap();
        
        match child.try_cast::<Player>() { 
            Ok(mut player) => { 
                player.call("set_as_local_player", &[]);
                self.local_player = Some(player);
                godot_print!("Local player configured from existing scene child");
            },
            Err(_) => godot_print!("Warning: No Player child found in MultiplayerManager"),
        }
    }

    fn handle_multiplayer_updates(&mut self, delta: f64) {
        if let Some(db_manager) = &self.db_manager {
            if let Err(e) = db_manager.tick() {
                godot_print!("Database tick error: {}", e);
            }

            // Send local player position updates
            self.position_update_timer += delta as f32;
            if self.position_update_timer >= self.position_update_interval {
                self.send_local_player_position();
                self.position_update_timer = 0.0;
            }

            // Update remote players based on database state
            self.sync_remote_players();
        }
    }

    fn send_local_player_position(&mut self) {
        if let Some(local_player) = &mut self.local_player {
            let position = local_player.call("get_player_position", &[]);
            if let Ok(pos) = position.try_to::<Vector2>() {
                if let Some(db_manager) = &self.db_manager {
                    if db_manager.is_logged_in() {
                        match db_manager.update_position(pos.x, pos.y) {
                            Ok(_) => {
                                // Position sent successfully
                            }
                            Err(e) => {
                                godot_print!("Failed to send position: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    fn sync_remote_players(&mut self) {
        // First, try to get our player ID if we don't have it yet
        if self.local_player_id.is_none() {
            if let Some(db_manager) = &self.db_manager {
                if let Some(connection) = db_manager.get_connection() {
                    if let Some(identity) = connection.try_identity() {
                        for player in connection.db.player().iter() {
                            if player.identity == identity {
                                self.local_player_id = Some(player.player_id);
                                break;
                            }
                        }
                    }
                }
            }
        }

        let (entities, players) = {
            if let Some(db_manager) = &self.db_manager {
                let entities = db_manager.get_all_entities();
                let players = db_manager.get_all_players();
                (entities, players)
            } else {
                return;
            }
        };

        // Create a map of entity_id -> player_name
        let player_names: HashMap<u32, String> = players.into_iter().collect();

        // Track active remote players
        let mut current_remote_players = std::collections::HashSet::new();

        // Process all entities (except our own)
        for (entity_id, x, y) in entities {
            if Some(entity_id) == self.local_player_id {
                continue; // Skip our own player
            }

            current_remote_players.insert(entity_id);
            let position = Vector2::new(x, y);

            if let Some(remote_player) = self.remote_players.get_mut(&entity_id) {
                // Update existing remote player position
                remote_player.call("set_player_position", &[position.to_variant()]);
            } else if let Some(player_name) = player_names.get(&entity_id) {
                // Spawn new remote player
                self.spawn_remote_player(entity_id, player_name, position);
            }
        }

        // Remove remote players who left
        let players_to_remove: Vec<u32> = self
            .remote_players
            .keys()
            .filter(|&player_id| !current_remote_players.contains(player_id))
            .cloned()
            .collect();

        for player_id in players_to_remove {
            self.remove_remote_player(player_id);
        }
    }

    fn spawn_remote_player(&mut self, player_id: u32, name: &str, position: Vector2) {
        godot_print!("Spawning remote player {} ({}) at ({}, {})", player_id, name, position.x, position.y);
        
        // Load the player scene
        let player_scene_path = "res://scenes/player.tscn";
        let mut resource_loader = ResourceLoader::singleton();
        
        if let Some(packed_scene) = resource_loader.load(player_scene_path) {
            if let Ok(scene) = packed_scene.try_cast::<PackedScene>() {
                if let Some(instance) = scene.instantiate() {
                    if let Ok(mut remote_player) = instance.try_cast::<Player>() {
                        // Configure the remote player
                        remote_player.set_position(position);
                        remote_player.set_name(&GString::from(format!("RemotePlayer_{}", player_id)));
                        remote_player.call("set_as_remote_player", &[]);
                        
                        // Add name label as child
                        let mut name_label = Label::new_alloc();
                        name_label.set_text(&GString::from(name));
                        name_label.set_position(Vector2::new(-30.0, -50.0));
                        remote_player.add_child(&name_label);
                        
                        // Add to scene
                        self.base_mut().add_child(&remote_player);
                        self.remote_players.insert(player_id, remote_player);
                        
                        godot_print!("Remote player {} spawned successfully", player_id);
                        return;
                    }
                }
            }
        }
        
        godot_print!("Failed to load player scene for remote player {}", player_id);
    }

    fn remove_remote_player(&mut self, player_id: u32) {
        if let Some(mut remote_player) = self.remote_players.remove(&player_id) {
            remote_player.queue_free();
            godot_print!("Removed remote player {}", player_id);
        }
    }
}
