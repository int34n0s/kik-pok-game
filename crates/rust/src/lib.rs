#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn
)]

mod characters;
mod errors;
mod multiplayer;
mod spacetimedb_client;
mod ui;
mod utils;
mod world;

pub use errors::RustLibError;

pub use characters::*;
pub use multiplayer::*;
pub use spacetimedb_client::*;
pub use ui::*;
pub use world::*;

use godot::prelude::*;

struct KikPokEngine;

#[gdextension]
unsafe impl ExtensionLibrary for KikPokEngine {}
