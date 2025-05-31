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

    if ctx.db.logged_out_player().identity().find(ctx.sender).is_some() {
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
pub fn register_player(
    ctx: &ReducerContext,
    name: String,
    scene_id: u32,
    direction: Direction,
) -> Result<(), String> {
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

    let positioning = Positioning {
        coordinates: scene.spawn_point,
        direction,
        in_on_floor: true,
    };

    match ctx.db.player().try_insert(Player {
        player_id: 0,
        identity: ctx.sender,
        name: name.trim().to_string(),
        positioning,
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
pub fn update_position(ctx: &ReducerContext, positioning: Positioning) -> Result<(), String> {
    let mut player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not registered")?;

    player.positioning = positioning;

    let player = ctx.db.player().identity().update(player);

    log::trace!(
        "Updated position for player {} to coordinates: ({}, {}), direction: {:?}, on_floor: {}",
        ctx.sender,
        player.positioning.coordinates.x,
        player.positioning.coordinates.y,
        player.positioning.direction,
        player.positioning.in_on_floor
    );

    Ok(())
}
