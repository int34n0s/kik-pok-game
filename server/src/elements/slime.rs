use crate::elements::DbVector2;

#[spacetimedb::table(name = green_slime, public)]
#[derive(Clone, Debug)]
pub struct GreenSlime {
    #[primary_key]
    #[auto_inc]
    pub green_slime_id: u64,

    pub position: DbVector2,

    pub scene_id: u32,
}

impl GreenSlime {
    pub fn new(position: DbVector2, scene_id: u32) -> Self {
        Self {
            green_slime_id: 0, // Auto-incremented
            position,
            scene_id,
        }
    }
}
