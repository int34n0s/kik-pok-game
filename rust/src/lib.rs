use godot::prelude::*;

mod characters;
mod ui;
mod multiplayer;

pub use multiplayer::*;
pub use ui::*;
pub use characters::*;

struct KikPokEngine;

#[gdextension]
unsafe impl ExtensionLibrary for KikPokEngine {}
