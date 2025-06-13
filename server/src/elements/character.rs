use crate::elements::utils::DbVector2;

use spacetimedb::Identity;

#[spacetimedb::table(name = player, public)]
#[spacetimedb::table(name = logged_out_player)]
#[derive(Debug, Clone)]
pub struct DbPlayer {
    #[primary_key]
    pub identity: Identity,

    #[unique]
    #[auto_inc]
    pub player_id: u32,

    pub name: String,

    pub position: DbVector2,

    pub direction: i32,
    pub is_jumping: bool,
}
