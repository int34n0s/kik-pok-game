use crate::elements::{player, world_scene, DBPlayerState, DbPlayer};
use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
pub fn register_player(ctx: &ReducerContext, name: String, scene_id: u32) -> Result<(), String> {
    log::trace!(
        "Player {} is registering with name: {} in scene: {}",
        ctx.sender,
        name,
        scene_id
    );

    if ctx.db.player().identity().find(ctx.sender).is_some() {
        return Err("Player already registered".to_string());
    }

    let scene = ctx
        .db
        .world_scene()
        .scene_id()
        .find(scene_id)
        .ok_or("Scene does not exist")?;

    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > 20 {
        return Err("Name too long (max 20 characters)".to_string());
    }

    match ctx.db.player().try_insert(DbPlayer {
        player_id: 0,
        identity: ctx.sender,
        name: name.trim().to_string(),
        state: DBPlayerState::with_position(scene.spawn_point),
    }) {
        Ok(player) => {
            log::info!(
                "Player {} registered successfully with name: {} and id: {} in scene: {}",
                player.identity,
                player.name,
                player.player_id,
                scene.name
            );
            Ok(())
        }
        Err(e) => {
            log::error!("Error registering player: {:?}", e);
            Err("Failed to register player".to_string())
        }
    }
}
