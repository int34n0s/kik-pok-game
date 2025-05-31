use crate::register_player_reducer::register_player;
use crate::update_position_reducer::update_position;
use crate::{DbConnection, ErrorContext, debug};
use crate::{EntityTableAccess, PlayerTableAccess};

use godot::prelude::*;

use spacetimedb_sdk::{Error, Table, credentials, DbContext};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    LoggedIn,
}

pub struct SpacetimeDBManager {
    connection: Option<DbConnection>,
    state: ConnectionState,
    instance_id: String,
}

impl Default for SpacetimeDBManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SpacetimeDBManager {
    pub fn new() -> Self {
        let instance_id = format!("kik-pok-{}", Uuid::new_v4());
        godot_print!(
            "Created SpacetimeDBManager with unique instance ID: {}",
            instance_id
        );

        Self {
            connection: None,
            state: ConnectionState::Disconnected,
            instance_id,
        }
    }

    pub fn connect(&mut self, host: &str, db_name: &str) -> Result<(), String> {
        if self.state != ConnectionState::Disconnected {
            return Err("Already connected or connecting".to_string());
        }

        godot_print!(
            "Connecting to SpacetimeDB at {} with database {}",
            host,
            db_name
        );

        self.state = ConnectionState::Connecting;

        match self.connect_to_db(&format!("http://{}", host), db_name) {
            Ok(connection) => {
                godot_print!("Successfully connected to SpacetimeDB");

                connection.reducers.on_debug(|_x| {
                    godot_print!("[debug]");
                });

                connection.reducers.on_register_player(|ctx, name| {
                    godot_print!("Player registration event: {} with status: {:?}", name, ctx.event.status);
                });

                // Subscribe to all tables to receive data
                connection.subscription_builder()
                    .subscribe("SELECT * FROM player");
                
                connection.subscription_builder()
                    .subscribe("SELECT * FROM entity");

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

    pub fn register_player(&mut self, username: String) -> Result<(), String> {
        if self.state != ConnectionState::Connected {
            return Err("Not connected to database".to_string());
        }

        let connection = self.connection.as_ref().ok_or("No connection available")?;

        godot_print!("Registering player with username: {}", username);

        match connection.reducers.register_player(username) {
            Ok(_) => {
                godot_print!("Player registration request sent successfully!");
                // For now, immediately set to LoggedIn - in production you'd wait for success callback
                self.state = ConnectionState::LoggedIn;
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to register player: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn update_position(&self, x: f32, y: f32) -> Result<(), String> {
        if self.state != ConnectionState::LoggedIn {
            return Err("Not logged in".to_string());
        }

        let connection = self.connection.as_ref().ok_or("No connection available")?;

        match connection.reducers.update_position(x, y) {
            Ok(_) => {
                // Don't spam logs with position updates
                Ok(())
            }
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

    pub fn get_all_players(&self) -> Vec<(u32, String)> {
        if let Some(connection) = &self.connection {
            connection
                .db()
                .player()
                .iter()
                .map(|player| (player.player_id, player.name.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_entities(&self) -> Vec<(u32, f32, f32)> {
        if let Some(connection) = &self.connection {
            connection
                .db
                .entity()
                .iter()
                .map(|entity| (entity.entity_id, entity.position.x, entity.position.y))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_player_by_entity_id(&self, entity_id: u32) -> Option<(u32, String)> {
        if let Some(connection) = &self.connection {
            connection
                .db
                .player()
                .iter()
                .find(|player| player.player_id == entity_id)
                .map(|player| (player.player_id, player.name.clone()))
        } else {
            None
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

    /// Load credentials from a file and connect to the database.
    fn connect_to_db(&self, host: &str, name: &str) -> Result<DbConnection, String> {
        let instance_id = self.instance_id.clone();
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

    pub fn call_debug(&self) -> Result<(), String> {
        godot_print!("Calling debug reducer...");

        godot_print!("Connection: {}", self.connection.is_some());

        // Call the debug reducer using the reducers field
        match self.connection.as_ref().map(|c| c.reducers.debug()) {
            Some(Ok(_)) => {
                godot_print!("Debug reducer called successfully!");

                Ok(())
            }
            Some(Err(e)) => {
                let error_msg = format!("Failed to call debug reducer: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
            None => {
                let error_msg = "No connection available".to_string();
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub fn tick(&self) -> Result<(), String> {
        match &self.connection {
            Some(connection) => connection.frame_tick().map_err(|e| e.to_string()),
            None => Err("No connection available".to_string()),
        }
    }
}
