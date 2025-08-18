use crate::elements::{character::player, world_scene::world_scene};
use crate::world_state::world_scene_config::WorldSceneConfig;

use spacetimedb::{reducer, ReducerContext, Table};

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!("Initializing...");

    WorldSceneConfig::initialize_all_scenes(ctx)?;

    Ok(())
}

#[reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!(
        "The identity_connected reducer was called by {}.",
        ctx.sender
    );

    if ctx.db.player().iter().any(|p| p.identity == ctx.sender) {
        return Err("Player already in the game".to_string());
    }

    Ok(())
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!(
        "The identity_disconnected reducer was called by {}.",
        ctx.sender
    );

    ctx.db.player().identity().delete(ctx.sender);

    Ok(())
}

#[reducer]
pub fn update_timestamp(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!("Updating timestamp...");

    let mut world_scene = ctx
        .db
        .world_scene()
        .iter()
        .find(|scene| scene.scene_id == 1)
        .ok_or("World scene not found")?;

    world_scene.last_update_time = ctx.timestamp;
    ctx.db.world_scene().scene_id().update(world_scene);

    Ok(())
}
