use crate::elements::{logged_out_player, player, world_scene, DbVector2, WorldScene};

use spacetimedb::{reducer, ReducerContext, Table};

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!("Initializing...");

    ctx.db.world_scene().insert(WorldScene::new(
        "Main".to_string(),
        DbVector2 { x: -15.0, y: -25.0 },
    ));

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

    if ctx
        .db
        .logged_out_player()
        .identity()
        .find(ctx.sender)
        .is_some()
    {
        ctx.db.logged_out_player().identity().delete(ctx.sender);
    }

    Ok(())
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!(
        "The identity_disconnected reducer was called by {}.",
        ctx.sender
    );

    if let Some(player) = ctx.db.player().identity().find(ctx.sender) {
        // Move player to logged_out_player table
        ctx.db.logged_out_player().insert(player);
        ctx.db.player().identity().delete(ctx.sender);
    }

    Ok(())
}
