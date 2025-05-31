use crate::elements::DbVector2;

#[spacetimedb::table(name = world_scene, public)]
#[derive(Debug, Clone)]
pub struct WorldScene {
    #[primary_key]
    #[auto_inc]
    pub scene_id: u32,

    pub name: String,

    pub spawn_point: DbVector2,
}

impl WorldScene {
    pub fn new(name: String, spawn_point: DbVector2) -> Self {
        Self {
            scene_id: 0,
            name,
            spawn_point,
        }
    }
}
