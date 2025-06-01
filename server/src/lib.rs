mod elements;

use elements::*;

use spacetimedb::{ReducerContext, Table};

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::trace!("Initializing...");

    ctx.db.world_scene().insert(WorldScene::new(
        "Main".to_string(),
        DbVector2 { x: -15.0, y: -25.0 },
    ));

    Ok(())
}

#[spacetimedb::reducer(client_connected)]
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

#[spacetimedb::reducer(client_disconnected)]
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

#[spacetimedb::reducer]
pub fn register_scene(
    ctx: &ReducerContext,
    scene_name: String,
    spawn_point: DbVector2,
) -> Result<(), String> {
    log::trace!("Registering scene with name: {}", scene_name);

    if scene_name.trim().is_empty() {
        return Err("Scene name cannot be empty".to_string());
    }

    if scene_name.len() > 50 {
        return Err("Scene name too long (max 50 characters)".to_string());
    }

    match ctx
        .db
        .world_scene()
        .try_insert(WorldScene::new(scene_name.trim().to_string(), spawn_point))
    {
        Ok(scene) => {
            log::info!(
                "Scene '{}' registered successfully with id: {}",
                scene.name,
                scene.scene_id
            );
            Ok(())
        }
        Err(e) => {
            log::error!("Error registering scene: {:?}", e);

            Err("Failed to register scene".to_string())
        }
    }
}

#[spacetimedb::reducer]
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
        positioning: scene.spawn_point,
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

#[spacetimedb::reducer]
pub fn update_position(ctx: &ReducerContext, positioning: DbVector2) -> Result<(), String> {
    let mut player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not registered")?;

    player.positioning = positioning;

    let _player = ctx.db.player().identity().update(player);

    log::trace!(
        "Updated position for player {} to coordinates: ({}, {})",
        ctx.sender,
        _player.positioning.x,
        _player.positioning.y,
    );

    Ok(())
}

#[spacetimedb::reducer]
pub fn register_coin(
    ctx: &ReducerContext,
    position: DbVector2,
    scene_id: u32,
) -> Result<(), String> {
    log::trace!(
        "Registering coin at position ({}, {}) in scene {}",
        position.x,
        position.y,
        scene_id
    );

    // Check if coin already exists at this position
    if ctx
        .db
        .coin()
        .iter()
        .any(|coin| coin.position.x == position.x && coin.position.y == position.y)
    {
        log::warn!(
            "Coin already exists at position ({}, {})",
            position.x,
            position.y
        );
        return Ok(()); // Don't error, just ignore duplicate registration
    }

    // Verify scene exists
    let _scene = ctx
        .db
        .world_scene()
        .scene_id()
        .find(scene_id)
        .ok_or("Scene does not exist")?;

    let coin = Coin {
        coin_id: 0, // Auto-incremented
        position,
        scene_id,
        is_collected: false,
        collected_by: None,
    };

    match ctx.db.coin().try_insert(coin) {
        Ok(inserted_coin) => {
            log::info!(
                "Coin registered at position ({}, {}) with id: {}",
                inserted_coin.position.x,
                inserted_coin.position.y,
                inserted_coin.coin_id
            );
            Ok(())
        }
        Err(e) => {
            log::error!("Error registering coin: {:?}", e);
            Err("Failed to register coin".to_string())
        }
    }
}

#[spacetimedb::reducer]
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
