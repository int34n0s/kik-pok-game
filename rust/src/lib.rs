mod characters;
mod multiplayer;
mod ui;
mod utils;
mod errors;

pub use errors::RustLibError;

pub use characters::*;
pub use multiplayer::*;
pub use ui::*;
pub use utils::*;

use godot::prelude::*;

struct KikPokEngine;

#[gdextension]
unsafe impl ExtensionLibrary for KikPokEngine {}
