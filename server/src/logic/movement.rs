use crate::elements::{player, DbVector2};

use spacetimedb::{reducer, ReducerContext};

#[reducer]
pub fn update_position(
    ctx: &ReducerContext,
    direction: i32,
    is_jumping: bool,
    position: DbVector2,
) -> Result<(), String> {
    let mut player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not registered")?;

    player.direction = direction;
    player.is_jumping = is_jumping;
    player.position = position;

    let _player = ctx.db.player().identity().update(player);

    log::trace!(
        "Updated position for player {} to direction: {} and is_jumping: {} and position: {:?}",
        ctx.sender,
        _player.direction,
        _player.is_jumping,
        _player.position,
    );

    Ok(())
}
