use crate::multiplayer::spacetimedb_client;
use crate::register_player_reducer::register_player;
use crate::update_position_reducer::update_position;

use crate::{
    CONNECTION_STATE, DbConnection, DbPlayer, ErrorContext, PlayerScoreTableAccess, collect_coin,
    register_coin,
};
use crate::{CoinTableAccess, DbVector2, PlayerTableAccess, WorldSceneTableAccess};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;

use godot::prelude::*;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use spacetimedb_sdk::{DbContext, Error, Table, credentials};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    LoggedIn,
}

#[derive(Default)]
pub struct SpacetimeDBManager {
    host: String,
    db_name: String,
    connection: Option<DbConnection>,
    state: ConnectionState,
    scene_id: Option<u32>,
    player_name: Option<String>,
}

impl SpacetimeDBManager {
    pub fn new(host: &str, db_name: &str) -> Self {
        godot_print!(
            "Connecting to SpacetimeDB at {} with database {}",
            host,
            db_name
        );

        let host = host.to_string();
        let db_name = db_name.to_string();

        // let connection = match Self::connect_to_db(&host, &db_name) {
        //     Ok(conn) => {
        //         conn.subscription_builder()
        //             .subscribe("SELECT * FROM player");
        //
        //         Some(conn)
        //     }
        //     Err(e) => {
        //         godot_error!("Couldn't connect to db {}: {}", db_name, e);
        //
        //         None
        //     }
        // };

        Self {
            host,
            db_name,
            connection: None,
            state: ConnectionState::Disconnected,
            scene_id: None,
            player_name: None,
        }
    }

    pub fn connect(&mut self, username: &str) -> Result<(), String> {
        if self.state != ConnectionState::Disconnected {
            return Err("Already connected or connecting".to_string());
        }

        // if self.connection.is_none() {
        //     godot_error!("Expected to be connected to SpacetimeDB at {}", self.host);
        //
        //     return Ok(());
        // }

        // if let Some(connection) = &self.connection {
        //     connection.disconnect().map_err(|e| e.to_string())?;
        // }

        self.state = ConnectionState::Connecting;
        self.player_name = Some(username.to_string());

        match self.connect_to_db_with_creds(username) {
            Ok(connection) => {
                godot_print!("Successfully connected to SpacetimeDB");

                connection
                    .reducers
                    .on_register_player(|ctx, _name, _scene_id| match &ctx.event.status {
                        spacetimedb_sdk::Status::Committed => {
                            godot_print!("Player registration committed successfully");
                            CONNECTION_STATE.store(1, Ordering::SeqCst);
                        }
                        spacetimedb_sdk::Status::Failed(e) => {
                            godot_print!("Player registration failed: {}", e);
                        }
                        spacetimedb_sdk::Status::OutOfEnergy => {
                            godot_print!("Player registration failed: Out of energy");
                        }
                    });

                // TODO: probably will move to the on_applied at some point
                connection
                    .subscription_builder()
                    .subscribe("SELECT * FROM player");

                connection
                    .subscription_builder()
                    .subscribe("SELECT * FROM world_scene");

                connection
                    .subscription_builder()
                    .subscribe("SELECT * FROM player_score");

                connection
                    .subscription_builder()
                    .subscribe("SELECT * FROM coin");

                connection.reducers.on_collect_coin(|_x, _x1| {
                    CONNECTION_STATE.store(2, Ordering::SeqCst);
                });

                self.connection = Some(connection);
                self.state = ConnectionState::Connected;

                Ok(())
            }
            Err(e) => {
                godot_print!("Failed to connect to SpacetimeDB: {}", e);
                self.state = ConnectionState::Disconnected;

                Err(e)
            }
        }
    }

    pub fn get_connection_mut(&mut self) -> Option<&mut DbConnection> {
        self.connection.as_mut()
    }

    fn connect_to_db_with_creds(&self, username: &str) -> Result<DbConnection, String> {
        let mut hasher = DefaultHasher::new();
        username.hash(&mut hasher);
        let seed = hasher.finish();

        let mut rng = StdRng::seed_from_u64(seed);
        let mut random_bytes = [0u8; 16];
        rng.fill_bytes(&mut random_bytes);

        let instance_id = Uuid::from_bytes(random_bytes).to_string();
        let creds_file = credentials::File::new(&instance_id);

        DbConnection::builder()
            .on_connect(move |_ctx, _identity, token| {
                let creds_store = credentials::File::new(&instance_id);
                if let Err(e) = creds_store.save(token) {
                    godot_print!("Failed to save credentials: {:?}", e);
                }
            })
            .on_connect_error(Self::on_connect_error)
            .on_disconnect(Self::on_disconnected)
            .with_token(creds_file.load().unwrap_or_default())
            .with_module_name(&self.db_name)
            .with_uri(&self.host)
            .build()
            .map_err(|e| e.to_string())
    }

    fn connect_to_db(host: &str, name: &str) -> Result<DbConnection, String> {
        let instance_id = Uuid::new_v4().to_string();
        let creds_file = credentials::File::new(&instance_id);

        DbConnection::builder()
            .on_connect_error(Self::on_connect_error)
            .on_disconnect(Self::on_disconnected)
            .with_token(creds_file.load().unwrap_or_default())
            .with_module_name(name)
            .with_uri(host)
            .build()
            .map_err(|e| e.to_string())
    }

    /// Our `on_connect_error` callback: print the error, then exit the process.
    fn on_connect_error(_ctx: &ErrorContext, err: Error) {
        godot_print!("Connection error: {:?}", err);
    }

    /// Our `on_disconnect` callback: print a note, then exit the process.
    fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
        if let Some(err) = err {
            godot_print!("Disconnected: {}", err);
        } else {
            godot_print!("Disconnected.");
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn is_logged_in(&self) -> bool {
        self.state == ConnectionState::LoggedIn
    }

    pub fn get_connection_state(&self) -> ConnectionState {
        self.state
    }
}

impl SpacetimeDBManager {
    pub fn register_player(&mut self, username: String, scene_id: u32) -> Result<(), String> {
        let connection = self.connection.as_ref().ok_or("No connection available")?;

        match connection.reducers.register_player(username, scene_id) {
            Ok(_) => {
                godot_print!("Player registration request sent successfully!");

                self.scene_id = Some(scene_id);

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to register player: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn send_inputs(
        &self,
        direction: i32,
        is_jumping: bool,
        position: Vector2,
    ) -> Result<(), String> {
        if self.state != ConnectionState::LoggedIn {
            return Err("Not logged in".to_string());
        }

        let connection = self.connection.as_ref().ok_or("No connection available")?;
        match connection.reducers.update_position(
            direction,
            is_jumping,
            DbVector2 {
                x: position.x,
                y: position.y,
            },
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = format!("Failed to update position: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn get_other_players(&self) -> Vec<DbPlayer> {
        if let Some(connection) = &self.connection {
            connection
                .db()
                .player()
                .iter()
                .filter(|x| x.identity != connection.identity())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_spawn_point(&self) -> Option<Vector2> {
        let scene_id = self.scene_id?;

        if let Some(connection) = &self.connection {
            connection
                .db()
                .world_scene()
                .iter()
                .find(|x| x.scene_id == scene_id)
                .map(|scene| Vector2::new(scene.spawn_point.x, scene.spawn_point.y))
        } else {
            None
        }
    }

    pub fn tick(&self) -> Result<(), String> {
        match &self.connection {
            Some(connection) => connection.frame_tick().map_err(|e| e.to_string()),
            None => Err("No connection available".to_string()),
        }
    }

    pub fn check_and_login(&mut self) -> bool {
        if CONNECTION_STATE.load(Ordering::SeqCst) != 0 {
            CONNECTION_STATE.store(0, Ordering::SeqCst);
            self.state = ConnectionState::LoggedIn;

            return true;
        }

        false
    }

    pub fn is_player_logged_in(&self, username: &str) -> bool {
        if let Some(connection) = &self.connection {
            connection
                .db()
                .player()
                .iter()
                .any(|player| player.name == username)
        } else {
            false
        }
    }

    pub fn get_player_name(&self) -> GString {
        self.player_name
            .as_ref()
            .map(|name| name.as_str().into())
            .unwrap_or_else(|| "Unknown".into())
    }

    pub fn collect_coin_at_position(&self, position: Vector2) -> Result<(), String> {
        if self.state != ConnectionState::LoggedIn {
            return Err("Not logged in".to_string());
        }

        let connection = self.connection.as_ref().ok_or("No connection available")?;
        let db_position = DbVector2 {
            x: position.x,
            y: position.y,
        };

        match connection.reducers.collect_coin(db_position) {
            Ok(_) => {
                godot_print!(
                    "Coin collection request sent successfully for position ({}, {})!",
                    position.x,
                    position.y
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to collect coin at ({}, {}): {}",
                    position.x, position.y, e
                );
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn register_coin_at_position(
        &self,
        position: Vector2,
        scene_id: u32,
    ) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }

        let connection = self.connection.as_ref().ok_or("No connection available")?;
        let db_position = DbVector2 {
            x: position.x,
            y: position.y,
        };

        match connection.reducers.register_coin(db_position, scene_id) {
            Ok(_) => {
                godot_print!(
                    "Coin registration request sent for position ({}, {})!",
                    position.x,
                    position.y
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to register coin at ({}, {}): {}",
                    position.x, position.y, e
                );
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn get_all_scores(&self) -> Vec<spacetimedb_client::player_score_type::PlayerScore> {
        if let Some(connection) = &self.connection {
            connection.db().player_score().iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_my_score(&self) -> u32 {
        if let Some(connection) = &self.connection {
            if let Some(score) = connection
                .db()
                .player_score()
                .iter()
                .find(|s| s.player_identity == connection.identity())
            {
                return score.coins_collected;
            }
        }
        0
    }

    pub fn get_all_coins(&self) -> Vec<spacetimedb_client::coin_type::Coin> {
        if let Some(connection) = &self.connection {
            connection.db().coin().iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn is_coin_collected_at_position(&self, position: Vector2) -> bool {
        if let Some(connection) = &self.connection {
            let db_position = DbVector2 {
                x: position.x,
                y: position.y,
            };
            connection
                .db()
                .coin()
                .iter()
                .find(|coin| coin.position.x == db_position.x && coin.position.y == db_position.y)
                .map(|coin| coin.is_collected)
                .unwrap_or(false)
        } else {
            false
        }
    }
}
