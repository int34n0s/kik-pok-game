use crate::{DbConnection, ErrorContext, RustLibError};

use godot::global::godot_print;

use spacetimedb_sdk::{Error, credentials};

use std::hash::{DefaultHasher, Hash, Hasher};

const DB_NAME: &str = "kik-pok";

#[cfg(feature = "remote-db")]
const DB_HOST: &str = "https://maincloud.spacetimedb.com";

#[cfg(not(feature = "remote-db"))]
const DB_HOST: &str = "127.0.0.1:3000";

#[derive(Default)]
pub struct ConnectionModule {
    connection: Option<DbConnection>,
}

impl ConnectionModule {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn connect(&mut self, username: &str) -> Result<(), RustLibError> {
        let jwt = Self::get_creds_store(username)
            .load()
            .map_err(|e| RustLibError::Credential { source: e })?;

        match self.connect_to_db_with_creds(jwt, Self::get_creds_store(username)) {
            Ok(connection) => {
                self.connection = Some(connection);

                Ok(())
            }
            Err(e) => {
                godot_print!("Connection failed (retry): {:?}", e);

                self.connection =
                    Some(self.connect_to_db_with_creds(None, Self::get_creds_store(username))?);

                Ok(())
            }
        }
    }

    pub fn get_connection(&self) -> Result<&DbConnection, RustLibError> {
        self.connection
            .as_ref()
            .ok_or(RustLibError::WrongConnectionState(
                "No connection established.".to_string(),
            ))
    }

    fn connect_to_db_with_creds(
        &self,
        jwt: Option<String>,
        creds_store: credentials::File,
    ) -> Result<DbConnection, RustLibError> {
        DbConnection::builder()
            .on_connect(move |_ctx, identity, token| {
                if let Err(e) = creds_store.save(token) {
                    godot_print!(
                        "Failed to save credentials: {:?}, for identity: {:?}",
                        e,
                        identity
                    );
                }
            })
            .on_connect_error(Self::on_connect_error)
            .on_disconnect(Self::on_disconnected)
            .with_token(jwt)
            .with_module_name(DB_NAME)
            .with_uri(DB_HOST)
            .build()
            .map_err(|e| RustLibError::SpacetimeSDK { source: e })
    }

    fn on_connect_error(_ctx: &ErrorContext, err: Error) {
        godot_print!("Connection error: {:?}", err);
    }

    fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
        if let Some(err) = err {
            godot_print!("Disconnected: {}", err);
        } else {
            godot_print!("Disconnected.");
        }
    }

    fn hash_username(username: &str) -> String {
        let mut hasher = DefaultHasher::new();
        username.hash(&mut hasher);
        hasher.finish().to_string()
    }

    fn get_creds_store(username: &str) -> credentials::File {
        credentials::File::new(Self::hash_username(username))
    }
}
