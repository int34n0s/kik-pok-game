use crate::multiplayer::connection_module::ConnectionModule;
use crate::register_player_reducer::register_player;

use crate::{send_player_state, try_collect_coin, CoinNode, ConnectionState, DbConnection, DbPlayer, DbPlayerState, GameManager, GreenSlimeNode, LocalPlayerNode, LoginModule, PlatformNode, RustLibError, WorldBootstrap};
use crate::{DbVector2, PlayerTableAccess, WorldSceneTableAccess};

use godot::prelude::*;

use spacetimedb_sdk::{DbContext, Error, Table};

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_CONNECTION: Arc<RwLock<SpacetimeDBManager>> =
        Arc::new(RwLock::new(SpacetimeDBManager::new()));

    pub static ref REGISTRATION_STATE: Arc<Mutex<RegistrationState>> =
    Arc::new(Mutex::new(RegistrationState::default()));
}

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
    pub fn new() -> Self {
        Self {
            connection_module: ConnectionModule::new(),
            login_module: LoginModule::new(),
        }
    }

    pub fn get_write_connection<'a>() -> Option<RwLockWriteGuard<'a, SpacetimeDBManager>> {
        if GLOBAL_CONNECTION.is_poisoned() {
            GLOBAL_CONNECTION.clear_poison();

            let mut connection = GLOBAL_CONNECTION.write().unwrap();
            *connection.login_module.get_state_mut() = ConnectionState::Disconnected;

            return None;
        }

        Some(GLOBAL_CONNECTION.write().unwrap())
    }

    pub fn get_read_connection<'a>() -> Option<RwLockReadGuard<'a, SpacetimeDBManager>> {
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

    fn register_subscribers(&mut self) -> Result<(), RustLibError> {
        let connection = self.connection_module.get_connection()?;

        CoinNode::setup_multiplayer(connection);
        GameManager::setup_multiplayer(connection);
        PlatformNode::setup_multiplayer(connection);
        GreenSlimeNode::setup_multiplayer(connection);
        WorldBootstrap::setup_multiplayer(connection);
        LocalPlayerNode::setup_multiplayer(connection, REGISTRATION_STATE.clone());

        Ok(())
    }
}

impl SpacetimeDBManager {
    pub fn tick(&mut self) -> Result<(), RustLibError> {
        if self.login_module.get_state() == &ConnectionState::Disconnected {
            return Ok(());
        }

        let connection = self.connection_module.get_connection()?;
        match connection
            .frame_tick() {
            Ok(_) => Ok(()),
            Err(e) => {
                match e {
                    Error::Disconnected => {
                        godot_print!("Disconnected from server");

                        *self.login_module.get_state_mut() = ConnectionState::Disconnected;

                        Ok(())  
                    },
                    _ => {
                        godot_print!("Error: {:?}", e);

                        Err(RustLibError::SpacetimeSDK { source: e })
                    }
                }
            }
        }
    }
}

impl SpacetimeDBManager {
    pub fn register_player(&mut self, username: String, scene_id: u32) -> Result<(), RustLibError> {
        let connection = self.connection_module.get_connection()?;
        match connection.reducers.register_player(username, scene_id) {
            Ok(_) => {
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
        let registration_state = REGISTRATION_STATE.lock().unwrap();
        match &*registration_state {
            RegistrationState::NotRegistered => {}
            RegistrationState::Registered => {
                *self.login_module.get_state_mut() = ConnectionState::LoggedIn;

                return true;
            }
            RegistrationState::RegistrationFailed(e) => {
                *self.login_module.get_state_mut() = ConnectionState::LoginFailed(e.clone());
            }
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
            Ok(_) => Ok(()),
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
            Ok(_) => Ok(()),
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
