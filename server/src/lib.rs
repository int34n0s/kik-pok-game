use spacetimedb::{Identity, SpacetimeType, ReducerContext, Table, Timestamp, TryInsertError};

// We're using this table as a singleton, so in this table
// there only be one element where the `id` is 0.
#[spacetimedb::table(name = config, public)]
pub struct Config {
    #[primary_key]
    pub id: u32,
    pub world_size: u64,
}

// This allows us to store 2D points in tables.
#[derive(SpacetimeType, Clone, Debug)]
pub struct DbVector2 {
    pub x: f32,
    pub y: f32,
}

#[spacetimedb::table(name = entity, public)]
#[derive(Debug, Clone)]
pub struct Entity {
    // The `auto_inc` attribute indicates to SpacetimeDB that
    // this value should be determined by SpacetimeDB on insert.
    #[auto_inc]
    #[primary_key]
    pub entity_id: u32,
    pub position: DbVector2,
    pub mass: u32,
}

#[spacetimedb::table(name = circle, public)]
pub struct Circle {
    #[primary_key]
    pub entity_id: u32,
    #[index(btree)]
    pub player_id: u32,
    pub direction: DbVector2,
    pub speed: f32,
    pub last_split_time: Timestamp,
}

#[spacetimedb::table(name = food, public)]
pub struct Food {
    #[primary_key]
    pub entity_id: u32,
}

#[spacetimedb::table(name = player, public)]
#[spacetimedb::table(name = logged_out_player)]
#[derive(Debug, Clone)]
pub struct Player {
    #[primary_key]
    identity: Identity,
    #[unique]
    #[auto_inc]
    player_id: u32,
    name: String,
}

#[spacetimedb::reducer]
pub fn debug(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("This reducer was called by {}.", ctx.sender);
    Ok(())
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Initializing...");
    
    ctx.db.config().try_insert(Config {
        id: 0,
        world_size: 1000,
    })?;

    Ok(())
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("The identity_connected reducer was called by {}.", ctx.sender);

    Ok(())
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("The identity_disconnected reducer was called by {}.", ctx.sender);

    if let Some(player) = ctx.db.player().identity().find(&ctx.sender) {
        // Remove player's entity if it exists
        ctx.db.entity().entity_id().delete(&player.player_id);
        
        // Move player to logged_out_player table
        ctx.db.logged_out_player().insert(player);
        ctx.db.player().identity().delete(&ctx.sender);
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn register_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    log::info!("Player {} is registering with name: {}", ctx.sender, name);
    
    // Check if player already exists
    if ctx.db.player().identity().find(&ctx.sender).is_some() {
        return Err("Player already registered".to_string());
    }
    
    // Validate name
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if name.len() > 20 {
        return Err("Name too long (max 20 characters)".to_string());
    }
    
    match ctx.db.player().try_insert(Player {
        identity: ctx.sender,
        player_id: 0, // auto_inc will set this
        name: name.trim().to_string(),
    }) {
        Ok(player) => log::info!("Player {} registered successfully with name: {} and id: {}", player.identity, player.name, player.player_id),
        Err(e) => log::error!("Error registering player: {:?}", e),
    }
    
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_position(ctx: &ReducerContext, x: f32, y: f32) -> Result<(), String> {
    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;
    
    if let Some(mut entity) = ctx.db.entity().entity_id().find(&player.player_id) {
        entity.position.x = x;
        entity.position.y = y;
        
        ctx.db.entity().entity_id().update(entity);
    } else {
        ctx.db.entity().try_insert(Entity {
            entity_id: player.player_id, // Use player_id as entity_id for simplicity
            position: DbVector2 { x, y },
            mass: 10,
        })?;
    }
    
    Ok(())
}
