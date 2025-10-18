use crate::elements::DbVector2;
use crate::elements::character::player;
use crate::elements::player_score::PlayerScore;
use crate::elements::{coin::coin, player_score::player_score};

use spacetimedb::{ReducerContext, Table, reducer};

#[reducer]
pub fn try_collect_coin(ctx: &ReducerContext, position: DbVector2) -> Result<(), String> {
    log::trace!(
        "Player {} is collecting a coin at position ({}, {})",
        ctx.sender,
        position.x,
        position.y
    );

    let player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not registered")?;

    let mut coin = ctx
        .db
        .coin()
        .iter()
        .find(|coin| coin.position.x == position.x && coin.position.y == position.y)
        .ok_or("Coin not found at this position")?;

    if coin.collected_by.is_some() {
        return Err("Coin already collected".to_string());
    }

    coin.collected_by = Some(ctx.sender);
    let updated_coin = ctx.db.coin().coin_id().update(coin);

    if let Some(mut score) = ctx.db.player_score().player_identity().find(ctx.sender) {
        score.add_coin();
        let updated_score = ctx.db.player_score().player_identity().update(score);

        log::info!(
            "Player {} ({}) collected coin at ({}, {})! New total: {} coins",
            player.name,
            ctx.sender,
            updated_coin.position.x,
            updated_coin.position.y,
            updated_score.coins_collected
        );
    } else {
        let scene_id = updated_coin.scene_id;

        let mut new_score = PlayerScore::new(ctx.sender, scene_id);
        new_score.add_coin();

        let inserted_score = ctx.db.player_score().insert(new_score);

        log::info!(
            "Player {} ({}) collected their first coin at ({}, {})! Score: {} coins",
            player.name,
            ctx.sender,
            updated_coin.position.x,
            updated_coin.position.y,
            inserted_score.coins_collected
        );
    }

    Ok(())
}
