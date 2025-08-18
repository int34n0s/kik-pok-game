use spacetimedb_sdk::Table;

use crate::{DbConnection, RustLibError, WorldScene, WorldSceneTableAccess};

pub fn get_diff_between_timestamps(world_scene: &WorldScene) -> i64 {
    let creation_time = world_scene
        .creation_time
        .to_time_duration_since_unix_epoch()
        .to_micros();
    let last_update_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64;

    last_update_time - creation_time
}

pub fn get_world_scene(connection: &DbConnection) -> Result<WorldScene, RustLibError> {
    connection
        .db
        .world_scene()
        .iter()
        .find(|x| x.scene_id == 1)
        .ok_or(RustLibError::WorldSetup("No platforms found".to_string()))
}
