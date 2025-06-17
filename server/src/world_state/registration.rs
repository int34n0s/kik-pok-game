use crate::elements::{coin, player, world_scene, Coin, DbPlayer, DbVector2, WorldScene};
use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
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
        position: scene.spawn_point,
        direction: 0,
        is_jumping: false,
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

#[reducer]
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
