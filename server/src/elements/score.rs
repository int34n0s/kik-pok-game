use spacetimedb::{Identity, SpacetimeType};
use crate::DbVector2;

#[spacetimedb::table(name = player_score, public)]
#[derive(Clone, Debug)]
pub struct PlayerScore {
    #[primary_key]
    #[auto_inc]
    pub score_id: u64,
    
    #[unique]
    pub player_identity: Identity,
    
    pub player_name: String,
    pub coins_collected: u32,
    pub scene_id: u32,
}

#[spacetimedb::table(name = coin, public)]
#[derive(Clone, Debug)]
pub struct Coin {
    #[primary_key]
    #[auto_inc]
    pub coin_id: u64,
    
    pub position: DbVector2,
    
    pub scene_id: u32,
    pub is_collected: bool,
    pub collected_by: Option<Identity>,
}

impl PlayerScore {
    pub fn new(player_identity: Identity, player_name: String, scene_id: u32) -> Self {
        Self {
            score_id: 0, // Auto-incremented
            player_identity,
            player_name,
            coins_collected: 0,
            scene_id,
        }
    }
    
    pub fn add_coin(&mut self) {
        self.coins_collected += 1;
    }
} 