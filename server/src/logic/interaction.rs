use crate::elements::{coin, player, player_score, DbVector2, PlayerScore};

use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
pub fn collect_coin(ctx: &ReducerContext, position: DbVector2) -> Result<(), String> {
    log::trace!(
        "Player {} is collecting a coin at position ({}, {})",
        ctx.sender,
        position.x,
        position.y
    );

    // Verify the player is registered
    let player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not registered")?;

    // Find the coin at this position
    let mut coin = ctx
        .db
        .coin()
        .iter()
        .find(|coin| coin.position.x == position.x && coin.position.y == position.y)
        .ok_or("Coin not found at this position")?;

    // Check if coin is already collected
    if coin.is_collected {
        return Err("Coin already collected".to_string());
    }

    // Mark coin as collected
    coin.is_collected = true;
    coin.collected_by = Some(ctx.sender);
    let updated_coin = ctx.db.coin().coin_id().update(coin);

    // Update player score
    if let Some(mut score) = ctx.db.player_score().player_identity().find(ctx.sender) {
        // Update existing score
        score.add_coin();
        let updated_score = ctx.db.player_score().player_identity().update(score);

        log::info!(
            "Player {} ({}) collected coin at ({}, {})! New total: {} coins",
            updated_score.player_name,
            ctx.sender,
            updated_coin.position.x,
            updated_coin.position.y,
            updated_score.coins_collected
        );
    } else {
        // Create new score record
        let scene_id = updated_coin.scene_id;

        let mut new_score = PlayerScore::new(ctx.sender, player.name.clone(), scene_id);
        new_score.add_coin();

        let inserted_score = ctx.db.player_score().insert(new_score);

        log::info!(
            "Player {} ({}) collected their first coin at ({}, {})! Score: {} coins",
            inserted_score.player_name,
            ctx.sender,
            updated_coin.position.x,
            updated_coin.position.y,
            inserted_score.coins_collected
        );
    }

    Ok(())
}
