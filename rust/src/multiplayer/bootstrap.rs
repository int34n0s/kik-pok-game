use crate::{CoinNode, CoinTableAccess, DbConnection, GreenSlimeNode, GreenSlimeTableAccess, LocalPlayerNode, MultiplayerManager, PlatformNode, PlatformTableAccess, RustLibError, WorldSceneTableAccess};

use godot::{prelude::*, obj::BaseMut};
use godot::classes::ResourceLoader;
use spacetimedb_sdk::{DbContext, Table};

const LOCAL_PLAYER_SCENE_PATH: &str = "res://scenes/characters/local_player.tscn";
const COIN_SCENE_PATH: &str = "res://scenes/entities/coin.tscn";
const PLATFORM_SCENE_PATH: &str = "res://scenes/environment/platform.tscn";
const ENEMY_SCENE_PATH: &str = "res://scenes/characters/green_slime.tscn";

pub struct WorldBootstrap {}

impl WorldBootstrap {
    pub fn new() -> WorldBootstrap {
        WorldBootstrap {}
    }

    pub fn setup_multiplayer(_connection: &DbConnection) {}

    pub fn start(
        &self,
        multiplayer_base: &mut BaseMut<MultiplayerManager>,
        connection: &DbConnection,
        player_name: &str,
    ) -> Result<(), RustLibError> {
        self.bootstrap_player(multiplayer_base, connection, player_name);
        self.bootstrap_coins(multiplayer_base, connection);
        self.bootstrap_platforms(multiplayer_base, connection);
        self.bootstrap_enemies(multiplayer_base, connection);

        Ok(())
    }

    fn bootstrap_player(&self, multiplayer_base: &mut BaseMut<MultiplayerManager>, connection: &DbConnection, _player_name: &str) {
        let player_id = connection.identity();
        
        // TODO: make it flexible
        let spawn_position = connection.db.world_scene().iter().find(|x| { x.scene_id == 1 }).unwrap().spawn_point;

        let mut resource_loader = ResourceLoader::singleton();
        let Some(packed_scene) = resource_loader.load(LOCAL_PLAYER_SCENE_PATH) else {
            godot_print!("Failed to load resource at {}", LOCAL_PLAYER_SCENE_PATH);
            return;
        };

        let Ok(scene) = packed_scene.try_cast::<PackedScene>() else {
            godot_print!("Failed to cast resource to PackedScene");
            return;
        };

        let Some(instance) = scene.instantiate() else {
            godot_print!("Failed to instantiate scene");
            return;
        };

        let Ok(mut local_player) = instance.try_cast::<LocalPlayerNode>() else {
            godot_print!("Failed to cast instance to Player");
            return;
        };

        local_player.set_position(spawn_position.into());
        local_player.set_name(&GString::from(player_id.to_string()));

        multiplayer_base.add_child(&local_player);
    }

    fn bootstrap_coins(&self, multiplayer_base: &mut BaseMut<MultiplayerManager>, connection: &DbConnection) {
        let coins = connection.db.coin().iter().filter(|x| x.scene_id == 1 && !x.is_collected).collect::<Vec<_>>();

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
    }
    
    fn bootstrap_platforms(&self, multiplayer_base: &mut BaseMut<MultiplayerManager>, connection: &DbConnection) {
        let platforms = connection.db.platform().iter().filter(|x| x.scene_id == 1).collect::<Vec<_>>();

        for platform in platforms {
            let mut resource_loader = ResourceLoader::singleton();
            let Some(packed_scene) = resource_loader.load(PLATFORM_SCENE_PATH) else {
                godot_print!("Failed to load resource at {}", PLATFORM_SCENE_PATH);
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
        
            let Ok(mut platform_node) = instance.try_cast::<PlatformNode>() else {
                godot_print!("Failed to cast instance to Platform");
                continue;
            };
        
            platform_node.set_position(platform.position.into());
            platform_node.set_name(&GString::from(platform.platform_id.to_string()));
        
            multiplayer_base.add_child(&platform_node);
        }
    }
    
    fn bootstrap_enemies(&self, multiplayer_base: &mut BaseMut<MultiplayerManager>, connection: &DbConnection) {
        let enemies = connection.db.green_slime().iter().filter(|x| x.scene_id == 1).collect::<Vec<_>>();

        for enemy in enemies {
            let mut resource_loader = ResourceLoader::singleton();
            let Some(packed_scene) = resource_loader.load(ENEMY_SCENE_PATH) else {
                godot_print!("Failed to load resource at {}", ENEMY_SCENE_PATH);
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
            
            let Ok(mut enemy_node) = instance.try_cast::<GreenSlimeNode>() else {
                godot_print!("Failed to cast instance to Enemy");
                continue;
            };

            enemy_node.set_position(enemy.position.into());
            enemy_node.set_name(&GString::from(enemy.green_slime_id.to_string()));

            multiplayer_base.add_child(&enemy_node);
        }
    }
}
