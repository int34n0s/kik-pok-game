use crate::elements::utils::DbVector2;

use spacetimedb::{Identity, SpacetimeType};

#[spacetimedb::table(name = player, public)]
#[derive(Debug, Clone)]
pub struct DbPlayer {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    #[auto_inc]
    pub player_id: u32,

    pub name: String,

    pub state: DBPlayerState,
}

#[derive(SpacetimeType, Debug, Clone)]
pub struct DBPlayerState {
    pub position: DbVector2,
    pub direction: i32,
    pub is_jumping: bool,
}

impl Default for DBPlayerState {
    fn default() -> Self {
        Self {
            position: DbVector2 { x: 0.0, y: 0.0 },
            direction: 0,
            is_jumping: false,
        }
    }
}

impl DBPlayerState {
    pub fn with_position(position: DbVector2) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }
}
