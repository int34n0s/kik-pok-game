use crate::spacetimedb_client::{DbConnection, ErrorContext, debug};
use godot::prelude::*;
use spacetimedb_sdk::{Error, Identity, credentials, DbContext};

pub struct SpacetimeDBManager {
    connection: DbConnection,
    // _thread_handle: std::thread::JoinHandle<()>,
}

impl SpacetimeDBManager {
    pub fn connect(host: &str, db_name: &str) -> Result<SpacetimeDBManager, String> {
        godot_print!(
            "Connecting to SpacetimeDB at {} with database {}",
            host,
            db_name
        );

        match Self::connect_to_db(&format!("http://{}", host), db_name) {
            Ok(connection) => {
                godot_print!("Successfully connected to SpacetimeDB");

                // let thread_handle = connection.run_threaded();
                connection.reducers.on_debug(|ctx| {
                    godot_print!("Debug reducer was called by: {:?}", ctx.event);
                });

                Ok(Self { 
                    connection,
                })
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Load credentials from a file and connect to the database.
    fn connect_to_db(host: &str, name: &str) -> Result<DbConnection, String> {
        DbConnection::builder()
            // Register our `on_connect` callback, which will save our auth token.
            .on_connect(Self::on_connected)
            // Register our `on_connect_error` callback, which will print a message, then exit the process.
            .on_connect_error(Self::on_connect_error)
            // Our `on_disconnect` callback, which will print a message, then exit the process.
            .on_disconnect(Self::on_disconnected)
            // If the user has previously connected, we'll have saved a token in the `on_connect` callback.
            // In that case, we'll load it and pass it to `with_token`,
            // so we can re-authenticate as the same `Identity`.
            .with_token(
                Self::creds_store()
                    .load()
                    .expect("Error loading credentials"),
            )
            // Set the database name we chose when we called `spacetime publish`.
            .with_module_name(name)
            // Set the URI of the SpacetimeDB host that's running our database.
            .with_uri(host)
            // Finalize configuration and connect!
            .build()
            .map_err(|e| e.to_string())
    }

    fn creds_store() -> credentials::File {
        credentials::File::new("quickstart-chat")
    }

    /// Our `on_connect` callback: save our credentials to a file.
    fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
        if let Err(e) = Self::creds_store().save(token) {
            eprintln!("Failed to save credentials: {:?}", e);
        }
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

        godot_print!("Connection: {}", self.connection.is_active());

        // Call the debug reducer using the reducers field
        match self.connection.reducers.debug() {
            Ok(_) => {
                self.connection.advance_one_message_blocking().map_err(|e| e.to_string())?;

                godot_print!("Debug reducer called successfully!");

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to call debug reducer: {}", e);
                godot_print!("{}", error_msg);
                Err(error_msg)
            }
        }
    }
}
