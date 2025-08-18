use crate::elements::character::{player, DBPlayerState};

use spacetimedb::{reducer, ReducerContext};

#[reducer]
pub fn send_player_state(ctx: &ReducerContext, state: DBPlayerState) -> Result<(), String> {
    let mut player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not registered")?;

    player.state = state;

    let _player = ctx.db.player().identity().update(player);

    log::trace!(
        "Updated position for player {} to {:?}",
        ctx.sender,
        _player.state
    );

    Ok(())
}
