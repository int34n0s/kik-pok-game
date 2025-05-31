use crate::PlayerTableAccess;
use crate::register_player_reducer::register_player;
use crate::update_position_reducer::update_position;
use crate::{CONNECTION_STATE, DbConnection, Direction, ErrorContext, Positioning};
use std::sync::atomic::Ordering;

use godot::prelude::*;

use crate::multiplayer::spacetimedb_client;
use spacetimedb_sdk::{DbContext, Error, Table, credentials};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    LoggedIn,
}

pub struct SpacetimeDBManager {
    pub connection: Option<DbConnection>,
    pub state: ConnectionState,
}

impl Default for SpacetimeDBManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SpacetimeDBManager {
    pub fn new() -> Self {
        Self {
            connection: None,
            state: ConnectionState::Disconnected,
        }
    }

    pub fn connect(&mut self, host: &str, db_name: &str, username: &str) -> Result<(), String> {
        if self.state != ConnectionState::Disconnected {
            return Err("Already connected or connecting".to_string());
        }

        godot_print!(
            "Connecting to SpacetimeDB at {} with database {}",
            host,
            db_name
        );

        self.state = ConnectionState::Connecting;

        match self.connect_to_db(&format!("http://{}", host), db_name, username) {
            Ok(connection) => {
                godot_print!("Successfully connected to SpacetimeDB");

                connection
                    .reducers
                    .on_register_player(|_ctx, name, scene_id, direction| {
                        godot_print!(
                            "Player registration event: {} in scene {} with direction {:?}",
                            name,
                            scene_id,
                            direction,
                        );

                        CONNECTION_STATE.store(1, Ordering::SeqCst);
                    });

                connection
                    .subscription_builder()
                    .subscribe("SELECT * FROM player");

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

    pub fn register_player(
        &mut self,
        username: String,
        scene_id: u32,
        direction: Direction,
    ) -> Result<(), String> {
        let connection = self.connection.as_ref().ok_or("No connection available")?;

        match connection
            .reducers
            .register_player(username, scene_id, direction)
        {
            Ok(_) => {
                godot_print!("Player registration request sent successfully!");

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to register player: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn update_position(&self, positioning: Positioning) -> Result<(), String> {
        if self.state != ConnectionState::LoggedIn {
            return Err("Not logged in".to_string());
        }

        let connection = self.connection.as_ref().ok_or("No connection available")?;

        match connection.reducers.update_position(positioning) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = format!("Failed to update position: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn get_connection(&self) -> Option<&DbConnection> {
        self.connection.as_ref()
    }

    pub fn get_other_players(&self) -> Vec<spacetimedb_client::player_type::Player> {
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

    pub fn get_connection_state(&self) -> ConnectionState {
        self.state
    }

    pub fn is_connected(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Connected | ConnectionState::LoggedIn
        )
    }

    pub fn is_logged_in(&self) -> bool {
        self.state == ConnectionState::LoggedIn
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    /// Load credentials from a file and connect to the database.
    fn connect_to_db(
        &self,
        host: &str,
        name: &str,
        username: &str,
    ) -> Result<DbConnection, String> {
        let instance_id = username.to_string();
        let creds_file = credentials::File::new(&instance_id);

        DbConnection::builder()
            // Register our `on_connect` callback, which will save our auth token.
            .on_connect(move |_ctx, _identity, token| {
                let creds_store = credentials::File::new(&instance_id);
                if let Err(e) = creds_store.save(token) {
                    eprintln!("Failed to save credentials: {:?}", e);
                }
            })
            // Register our `on_connect_error` callback, which will print a message, then exit the process.
            .on_connect_error(Self::on_connect_error)
            // Our `on_disconnect` callback, which will print a message, then exit the process.
            .on_disconnect(Self::on_disconnected)
            // If the user has previously connected, we'll have saved a token in the `on_connect` callback.
            // In that case, we'll load it and pass it to `with_token`,
            // so we can re-authenticate as the same `Identity`.
            .with_token(
                creds_file.load().unwrap_or_default(), // Use empty string if no credentials exist yet
            )
            // Set the database name we chose when we called `spacetime publish`.
            .with_module_name(name)
            // Set the URI of the SpacetimeDB host that's running our database.
            .with_uri(host)
            // Finalize configuration and connect!
            .build()
            .map_err(|e| e.to_string())
    }

    /// Our `on_connect_error` callback: print the error, then exit the process.
    fn on_connect_error(_ctx: &ErrorContext, err: Error) {
        eprintln!("Connection error: {:?}", err);
        std::process::exit(1);
    }

    /// Our `on_disconnect` callback: print a note, then exit the process.
    fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
        if let Some(err) = err {
            eprintln!("Disconnected: {}", err);
            std::process::exit(1);
        } else {
            println!("Disconnected.");
            std::process::exit(0);
        }
    }

    pub fn tick(&self) -> Result<(), String> {
        match &self.connection {
            Some(connection) => connection.frame_tick().map_err(|e| e.to_string()),
            None => Err("No connection available".to_string()),
        }
    }
}
