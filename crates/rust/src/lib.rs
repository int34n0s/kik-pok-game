mod characters;
mod errors;
mod multiplayer;
mod spacetimedb_client;
mod ui;
mod utils;

pub use errors::RustLibError;

pub use characters::*;
pub use multiplayer::*;
pub use spacetimedb_client::*;
pub use ui::*;

use godot::prelude::*;

struct KikPokEngine;

#[gdextension]
unsafe impl ExtensionLibrary for KikPokEngine {}
