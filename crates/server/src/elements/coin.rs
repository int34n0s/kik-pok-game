use crate::elements::DbVector2;
use spacetimedb::Identity;

#[spacetimedb::table(name = coin, public)]
#[derive(Clone, Debug)]
pub struct Coin {
    #[primary_key]
    #[auto_inc]
    pub coin_id: u64,

    pub position: DbVector2,

    pub scene_id: u32,

    pub collected_by: Option<Identity>,
}
