use crate::elements::{coin, world_scene, Coin, DbVector2, WorldScene};

use itertools::Itertools;
use spacetimedb::{ReducerContext, Table};

pub struct WorldSceneConfig {
    name: &'static str,
    spawn_point: DbVector2,
    coin_positions: &'static [DbVector2],
}

impl WorldSceneConfig {
    const SCENES: &'static [WorldSceneConfig] = &[WorldSceneConfig {
        name: "Main",
        spawn_point: DbVector2 { x: -15.0, y: -25.0 },
        coin_positions: &[
            DbVector2 { x: 10.0, y: -30.0 },
            DbVector2 { x: 25.0, y: -35.0 },
            DbVector2 { x: -5.0, y: -20.0 },
            DbVector2 { x: 40.0, y: -45.0 },
        ],
    }];

    pub fn initialize_all_scenes(ctx: &ReducerContext) -> Result<(), String> {
        for scene_config in Self::SCENES {
            ctx.db.world_scene().insert(WorldScene::new(
                scene_config.name.to_string(),
                scene_config.spawn_point.clone(),
            ));
            
            Self::initialize_coins(ctx, scene_config)?;

            log::info!("Initialized scene: {}", scene_config.name);
        }

        Ok(())
    }

    fn initialize_coins(
        ctx: &ReducerContext,
        scene_config: &WorldSceneConfig,
    ) -> Result<(), String> {
        let world_scene = ctx
            .db
            .world_scene()
            .iter()
            .find(|scene| scene.name == scene_config.name)
            .ok_or("Scene does not exist")?;
        
        let unique_positions = scene_config
            .coin_positions
            .iter()
            .unique()
            .collect::<Vec<_>>();

        if unique_positions.is_empty() {
            return Ok(());
        }

        for position in unique_positions {
            let coin = Coin {
                coin_id: 0,
                position: position.clone(),
                scene_id: world_scene.scene_id,
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
                }
                Err(e) => {
                    log::error!("Error registering coin: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
