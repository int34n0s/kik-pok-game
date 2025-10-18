use spacetimedb::{ReducerContext, Table, Timestamp};

use crate::elements::DbVector2;

#[spacetimedb::table(name = world_scene, public)]
#[derive(Debug, Clone)]
pub struct WorldScene {
    #[primary_key]
    #[auto_inc]
    pub scene_id: u32,

    pub name: String,
    pub creation_time: Timestamp,
    pub last_update_time: Timestamp,

    pub spawn_point: DbVector2,
}

impl WorldScene {
    pub fn new(name: String, spawn_point: DbVector2, creation_time: Timestamp) -> Self {
        Self {
            scene_id: 0,
            name,
            creation_time,
            last_update_time: creation_time,
            spawn_point,
        }
    }

    pub fn set_creation_time(ctx: &ReducerContext, creation_time: Timestamp) -> Result<(), String> {
        let mut world_scene = ctx
            .db
            .world_scene()
            .iter()
            .find(|scene| scene.scene_id == 1)
            .ok_or("World scene not found")?;

        world_scene.creation_time = creation_time;
        world_scene.last_update_time = creation_time;
        ctx.db.world_scene().scene_id().update(world_scene);

        Ok(())
    }
}
