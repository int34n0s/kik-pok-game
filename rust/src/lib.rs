mod characters;
mod multiplayer;
mod ui;
mod utils;

pub use characters::*;
pub use multiplayer::*;
pub use ui::*;
pub use utils::*;

use std::sync::Mutex;
use std::sync::atomic::AtomicU8;

use godot::prelude::*;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_CONNECTION: Mutex<SpacetimeDBManager> =
        Mutex::new(SpacetimeDBManager::new());
    pub static ref CONNECTION_STATE: AtomicU8 = AtomicU8::new(0);
}

struct KikPokEngine;

#[gdextension]
unsafe impl ExtensionLibrary for KikPokEngine {}
