use godot::prelude::*;

pub mod player;
pub mod spacetimedb_client;
pub mod spacetimedb_manager;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}