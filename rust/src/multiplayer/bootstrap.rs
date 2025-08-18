use crate::{
    CoinNode, CoinTableAccess, DbConnection, GreenSlimeNode, LocalPlayerNode, MultiplayerManager,
    PlatformNode, RustLibError, WorldSceneTableAccess, get_diff_between_timestamps,
    get_world_scene, update_timestamp,
};

use godot::classes::{PackedScene, ResourceLoader};
use godot::{obj::BaseMut, prelude::*};
use spacetimedb_sdk::{DbContext, Table};

const COIN_SCENE_PATH: &str = "res://scenes/entities/coin.tscn";
const LOCAL_PLAYER_SCENE_PATH: &str = "res://scenes/characters/local_player.tscn";

pub struct WorldBootstrap {}

impl Default for WorldBootstrap {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldBootstrap {
    pub fn new() -> WorldBootstrap {
        WorldBootstrap {}
    }

    pub fn setup_multiplayer(connection: &DbConnection) {
        connection
            .subscription_builder()
            .subscribe("SELECT * FROM world_scene");
    }

    pub fn boot_player(
        &self,
        multiplayer_base: &mut BaseMut<MultiplayerManager>,
        connection: &DbConnection,
        player_name: &str,
    ) -> Result<(), RustLibError> {
        self.bootstrap_coins(multiplayer_base, connection)?;
        self.bootstrap_player(multiplayer_base, connection, player_name)?;

        self.sync_platforms(multiplayer_base, connection)?;
        self.sync_animated_enemies(multiplayer_base, connection)?;

        Ok(())
    }

    fn bootstrap_player(
        &self,
        multiplayer_base: &mut BaseMut<MultiplayerManager>,
        connection: &DbConnection,
        _player_name: &str,
    ) -> Result<(), RustLibError> {
        let player_id = connection.identity();

        // TODO: make it flexible
        let spawn_position = connection
            .db
            .world_scene()
            .iter()
            .find(|x| x.scene_id == 1)
            .ok_or(RustLibError::WorldSetup(
                "No spawn position found".to_string(),
            ))?
            .spawn_point;

        let mut resource_loader = ResourceLoader::singleton();
        let Some(packed_scene) = resource_loader.load(LOCAL_PLAYER_SCENE_PATH) else {
            return Err(RustLibError::WorldSetup(format!(
                "Failed to load resource at {}",
                LOCAL_PLAYER_SCENE_PATH
            )));
        };

        let Ok(scene) = packed_scene.try_cast::<PackedScene>() else {
            return Err(RustLibError::WorldSetup(
                "Failed to cast resource to PackedScene".to_string(),
            ));
        };

        let Some(instance) = scene.instantiate() else {
            return Err(RustLibError::WorldSetup(
                "Failed to instantiate scene".to_string(),
            ));
        };

        let Ok(mut local_player) = instance.try_cast::<LocalPlayerNode>() else {
            return Err(RustLibError::WorldSetup(
                "Failed to cast instance to Player".to_string(),
            ));
        };

        local_player.set_position(spawn_position.into());
        local_player.set_name(&GString::from(player_id.to_string()));

        multiplayer_base.add_child(&local_player);

        Ok(())
    }

    fn bootstrap_coins(
        &self,
        multiplayer_base: &mut BaseMut<MultiplayerManager>,
        connection: &DbConnection,
    ) -> Result<(), RustLibError> {
        let coins = connection
            .db
            .coin()
            .iter()
            .filter(|x| x.scene_id == 1 && x.collected_by.is_none())
            .collect::<Vec<_>>();

        for coin in coins {
            let mut resource_loader = ResourceLoader::singleton();
            let Some(packed_scene) = resource_loader.load(COIN_SCENE_PATH) else {
                godot_print!("Failed to load resource at {}", COIN_SCENE_PATH);
                continue;
            };

            let Ok(scene) = packed_scene.try_cast::<PackedScene>() else {
                godot_print!("Failed to cast resource to PackedScene");
                continue;
            };

            let Some(instance) = scene.instantiate() else {
                godot_print!("Failed to instantiate scene");
                continue;
            };

            let Ok(mut coin_node) = instance.try_cast::<CoinNode>() else {
                godot_print!("Failed to cast instance to Coin");
                continue;
            };

            coin_node.set_position(coin.position.into());
            coin_node.set_name(&GString::from(coin.coin_id.to_string()));

            multiplayer_base.add_child(&coin_node);
        }

        Ok(())
    }

    fn sync_platforms(
        &self,
        multiplayer_base: &mut BaseMut<MultiplayerManager>,
        connection: &DbConnection,
    ) -> Result<(), RustLibError> {
        if let Err(e) = connection.reducers.update_timestamp() {
            godot_print!("Failed to update timestamp: {}", e);
        }

        let platform_group = multiplayer_base
            .try_get_node_as::<Node>("Platforms")
            .unwrap();
        let platform_count = platform_group.get_child_count();

        for i in 1..=platform_count {
            let platform =
                platform_group.try_get_node_as::<PlatformNode>(format!("Platform{}", i).as_str());

            match connection.reducers.update_timestamp() {
                Ok(_) => {}
                Err(e) => godot_print!("Failed to update timestamp: {}", e),
            }

            match connection.frame_tick() {
                Ok(_) => {
                    let world_scene = get_world_scene(connection)?;
                    let t_micro = get_diff_between_timestamps(&world_scene) as f64;

                    if let Some(mut platform) = platform {
                        platform.bind_mut().sync_based_on_time(t_micro);
                    }
                }
                Err(e) => godot_print!("Database tick error: {}", e),
            }
        }

        Ok(())
    }

    fn sync_animated_enemies(
        &self,
        multiplayer_base: &mut BaseMut<MultiplayerManager>,
        connection: &DbConnection,
    ) -> Result<(), RustLibError> {
        if let Err(e) = connection.reducers.update_timestamp() {
            godot_print!("Failed to update timestamp: {}", e);
        }

        let enemy_group = multiplayer_base.try_get_node_as::<Node>("Enemies").unwrap();
        let enemy_count = enemy_group.get_child_count();

        for i in 1..=enemy_count {
            let enemy =
                enemy_group.try_get_node_as::<GreenSlimeNode>(format!("Enemy{}", i).as_str());

            match connection.reducers.update_timestamp() {
                Ok(_) => {}
                Err(e) => godot_print!("Failed to update timestamp: {}", e),
            }

            match connection.frame_tick() {
                Ok(_) => {
                    let world_scene = get_world_scene(connection)?;
                    let t_micro = get_diff_between_timestamps(&world_scene) as f64;

                    if let Some(mut enemy) = enemy {
                        enemy.bind_mut().sync_based_on_time(t_micro);
                    }
                }
                Err(e) => godot_print!("Database tick error: {}", e),
            }
        }

        Ok(())
    }
}
