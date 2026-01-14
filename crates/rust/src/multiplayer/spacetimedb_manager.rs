use crate::multiplayer::connection_module::ConnectionModule;
use crate::register_player_reducer::register_player;

use crate::{
    CoinNode, ConnectionState, DbConnection, DbPlayer, DbPlayerState, GreenSlimeNode,
    LocalPlayerNode, LoginModule, PlatformNode, RustLibError, StatusPanel, WorldBootstrap,
    send_player_state, try_collect_coin,
};
use crate::{DbVector2, PlayerTableAccess, WorldSceneTableAccess};

use godot::prelude::*;

use spacetimedb_sdk::{DbContext, Error, Table};

use std::sync::{Arc, LazyLock, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

static GLOBAL_CONNECTION: LazyLock<Arc<RwLock<SpacetimeDBManager>>> =
    LazyLock::new(|| Arc::new(RwLock::new(SpacetimeDBManager::new())));

pub static REGISTRATION_STATE: LazyLock<Arc<Mutex<RegistrationState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(RegistrationState::default())));

#[derive(Default)]
pub enum RegistrationState {
    #[default]
    NotRegistered,
    Registered,
    RegistrationFailed(String),
}

#[derive(Default)]
pub struct SpacetimeDBManager {
    pub login_module: LoginModule,
    connection_module: ConnectionModule,
}

impl SpacetimeDBManager {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new() -> Self {
        Self {
            connection_module: ConnectionModule::new(),
            login_module: LoginModule::new(),
        }
    }

    pub fn get_write_connection<'a>() -> Option<RwLockWriteGuard<'a, Self>> {
        if GLOBAL_CONNECTION.is_poisoned() {
            GLOBAL_CONNECTION.clear_poison();

            let mut connection = GLOBAL_CONNECTION.write().unwrap();
            *connection.login_module.get_state_mut() = ConnectionState::Disconnected;

            return None;
        }

        Some(GLOBAL_CONNECTION.write().unwrap())
    }

    pub fn get_read_connection<'a>() -> Option<RwLockReadGuard<'a, Self>> {
        if GLOBAL_CONNECTION.is_poisoned() {
            GLOBAL_CONNECTION.clear_poison();

            let mut connection = GLOBAL_CONNECTION.write().unwrap();
            *connection.login_module.get_state_mut() = ConnectionState::Disconnected;

            return None;
        }

        Some(GLOBAL_CONNECTION.read().unwrap())
    }

    pub fn connect(&mut self, username: &str) -> Result<(), RustLibError> {
        self.connect_to_server(username)?;
        self.register_subscribers()?;

        self.login_module.set_scene_id(1);
        self.login_module.set_player_name(username.to_string());

        *self.login_module.get_state_mut() = ConnectionState::Connected;

        Ok(())
    }

    pub fn get_connection(&self) -> Result<&DbConnection, RustLibError> {
        self.connection_module.get_connection()
    }

    fn connect_to_server(&mut self, username: &str) -> Result<(), RustLibError> {
        self.connection_module.connect(username)
    }

    fn register_subscribers(&self) -> Result<(), RustLibError> {
        let connection = self.connection_module.get_connection()?;

        CoinNode::setup_multiplayer(connection);
        StatusPanel::setup_multiplayer(connection);
        PlatformNode::setup_multiplayer(connection);
        GreenSlimeNode::setup_multiplayer(connection);
        WorldBootstrap::setup_multiplayer(connection);
        LocalPlayerNode::setup_multiplayer(connection, &REGISTRATION_STATE);

        Ok(())
    }
}

impl SpacetimeDBManager {
    pub fn tick(&mut self) -> Result<(), RustLibError> {
        if self.login_module.get_state() == &ConnectionState::Disconnected {
            return Ok(());
        }

        let connection: &DbConnection = self.connection_module.get_connection()?;

        match connection.frame_tick() {
            Ok(()) => Ok(()),
            Err(e) => {
                if matches!(e, Error::Disconnected) {
                    godot_print!("Disconnected from server");
                    *self.login_module.get_state_mut() = ConnectionState::Disconnected;
                    Ok(())
                } else {
                    godot_print!("Error: {:?}", e);
                    Err(RustLibError::SpacetimeSDK { source: e })
                }
            }
        }
    }
}

impl SpacetimeDBManager {
    pub fn register_player(&mut self, username: String, scene_id: u32) -> Result<(), RustLibError> {
        let connection = self.connection_module.get_connection()?;
        match connection.reducers.register_player(username, scene_id) {
            Ok(()) => {
                godot_print!("Player registration request sent successfully!");

                Ok(())
            }
            Err(e) => {
                godot_print!("Failed to register player: {}", e);

                Err(RustLibError::SpacetimeSDK { source: e })
            }
        }
    }

    pub fn get_spawn_point(&self) -> Result<Option<Vector2>, RustLibError> {
        let connection = self.connection_module.get_connection()?;
        let scene_id = self
            .login_module
            .get_scene_id()
            .ok_or(RustLibError::WorldSetup(
                "Expected scene id to be in the Login Module.".to_string(),
            ))?;

        Ok(connection
            .db()
            .world_scene()
            .iter()
            .find(|x| x.scene_id == scene_id)
            .map(|scene| Vector2::new(scene.spawn_point.x, scene.spawn_point.y)))
    }

    pub fn check_and_login(&mut self) -> bool {
        let state = {
            let registration_state = REGISTRATION_STATE.lock().unwrap();
            match &*registration_state {
                RegistrationState::NotRegistered => None,
                RegistrationState::Registered => Some(ConnectionState::LoggedIn),
                RegistrationState::RegistrationFailed(e) => {
                    Some(ConnectionState::LoginFailed(e.clone()))
                }
            }
        };

        if let Some(new_state) = state {
            let logged_in = new_state == ConnectionState::LoggedIn;
            *self.login_module.get_state_mut() = new_state;
            return logged_in;
        }

        false
    }

    pub fn is_player_logged_in(&self, username: &str) -> bool {
        let Ok(connection) = self.connection_module.get_connection() else {
            return false;
        };

        connection
            .db()
            .player()
            .iter()
            .any(|player| player.name == username)
    }

    pub fn get_other_players(&self) -> Result<Vec<DbPlayer>, RustLibError> {
        let connection = self.connection_module.get_connection()?;
        Ok(connection
            .db()
            .player()
            .iter()
            .filter(|x| x.identity != connection.identity())
            .collect())
    }
}

impl SpacetimeDBManager {
    pub fn send_inputs(&self, state: DbPlayerState) -> Result<(), RustLibError> {
        self.login_module.require_logged_in()?;

        let connection = self.connection_module.get_connection()?;
        match connection.reducers.send_player_state(state) {
            Ok(()) => Ok(()),
            Err(e) => {
                godot_print!("Failed to update position: {}", e);

                Err(RustLibError::SpacetimeSDK { source: e })
            }
        }
    }

    pub fn collect_coin_at_position(&self, position: Vector2) -> Result<(), RustLibError> {
        self.login_module.require_logged_in()?;

        let connection = self.connection_module.get_connection()?;
        let db_position = DbVector2 {
            x: position.x,
            y: position.y,
        };

        match connection.reducers.try_collect_coin(db_position) {
            Ok(()) => Ok(()),
            Err(e) => {
                godot_print!(
                    "Failed to collect coin at ({}, {}): {}",
                    position.x,
                    position.y,
                    e
                );

                Err(RustLibError::SpacetimeSDK { source: e })
            }
        }
    }
}
