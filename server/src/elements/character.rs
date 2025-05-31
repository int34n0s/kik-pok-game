use crate::elements::utils::DbVector2;
use crate::elements::Direction;

use spacetimedb::{Identity, SpacetimeType};

#[spacetimedb::table(name = player, public)]
#[spacetimedb::table(name = logged_out_player)]
#[derive(Debug, Clone)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    #[auto_inc]
    pub player_id: u32,
    pub name: String,
    pub positioning: Positioning,
}

#[derive(SpacetimeType, Debug, Clone, Default)]
pub struct Positioning {
    pub coordinates: DbVector2,

    pub direction: Direction,

    pub in_on_floor: bool,
}
