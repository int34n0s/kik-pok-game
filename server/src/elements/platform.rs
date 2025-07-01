use crate::elements::DbVector2;

#[spacetimedb::table(name = platform, public)]
#[derive(Clone, Debug)]
pub struct Platform {
    #[primary_key]
    #[auto_inc]
    pub platform_id: u64,

    pub position: DbVector2,

    pub scene_id: u32,
}

impl Platform {
    pub fn new(position: DbVector2, scene_id: u32) -> Self {
        Self {
            platform_id: 0, // Auto-incremented
            position,
            scene_id,
        }
    }
}
