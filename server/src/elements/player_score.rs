use spacetimedb::Identity;

#[spacetimedb::table(name = player_score, public)]
#[derive(Clone, Debug)]
pub struct PlayerScore {
    #[primary_key]
    #[auto_inc]
    pub score_id: u64,

    #[unique]
    pub player_identity: Identity,

    pub coins_collected: u32,

    pub scene_id: u32,
}

impl PlayerScore {
    pub fn new(player_identity: Identity, scene_id: u32) -> Self {
        Self {
            score_id: 0, // Auto-incremented
            player_identity,
            coins_collected: 0,
            scene_id,
        }
    }

    pub fn add_coin(&mut self) {
        self.coins_collected += 1;
    }
}
