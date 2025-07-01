use crate::elements::{DbVector2};
use crate::elements::coin::{coin, Coin};
use crate::elements::platform::{platform, Platform};
use crate::elements::scene::{world_scene, WorldScene};
use crate::elements::slime::{green_slime, GreenSlime};

use itertools::Itertools;
use spacetimedb::{ReducerContext, Table};

pub struct WorldSceneConfig {
    name: &'static str,
    spawn_point: DbVector2,
    coin_positions: &'static [DbVector2],
    platform_positions: &'static [DbVector2],
    green_slime_positions: &'static [DbVector2],
}

impl WorldSceneConfig {
    const SCENES: &'static [WorldSceneConfig] = &[WorldSceneConfig {
        name: "Main",
        spawn_point: DbVector2 { x: -15.0, y: -35.0 },
        coin_positions: &[
            DbVector2 { x: 60.0, y: -25.0 },
            DbVector2 { x: 98.0, y: -25.0 },
            DbVector2 { x: 178.0, y: -25.0 },
            DbVector2 { x: 178.0, y: -120.0 },
            DbVector2 { x: 498.0, y: -104.0 },
            DbVector2 { x: 530.0, y: -88.0 },
            DbVector2 { x: 690.0, y: -104.0 },
            DbVector2 { x: 626.0, y: -344.0 },
            DbVector2 { x: 642.0, y: -328.0 },
            DbVector2 { x: 674.0, y: -312.0 },
            DbVector2 { x: 834.0, y: -312.0 },
            DbVector2 { x: 882.0, y: -312.0 },
            DbVector2 { x: 754.0, y: -296.0 },
            DbVector2 { x: 784.0, y: -296.0 },
            DbVector2 { x: 834.0, y: 23.0 },
        ],
        platform_positions: &[
            DbVector2 { x: 13.0, y: -60.0 },
            DbVector2 { x: 299.0, y: -77.0 },
            DbVector2 { x: 923.0, y: 67.0 },
            DbVector2 { x: 1153.0, y: 35.0 },
            DbVector2 { x: 1189.0, y: -36.0 },
            DbVector2 { x: 1190.0, y: -93.0 },
            DbVector2 { x: 1035.0, y: -151.0 },
        ],
        green_slime_positions: &[
            DbVector2 { x: 562.0, y: -78.0 },
        ],
    }];

    pub fn initialize_all_scenes(ctx: &ReducerContext) -> Result<(), String> {
        for scene_config in Self::SCENES {
            ctx.db.world_scene().insert(WorldScene::new(
                scene_config.name.to_string(),
                scene_config.spawn_point.clone(),
            ));

            Self::initialize_coins(ctx, scene_config)?;
            Self::initialize_platforms(ctx, scene_config)?;
            Self::initialize_green_slimes(ctx, scene_config)?;

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

        let coin_unique_positions = scene_config
            .coin_positions
            .iter()
            .unique()
            .collect::<Vec<_>>();

        if coin_unique_positions.is_empty() {
            return Ok(());
        }

        for position in coin_unique_positions {
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

    fn initialize_platforms(
        ctx: &ReducerContext,
        scene_config: &WorldSceneConfig,
    ) -> Result<(), String> {
        let world_scene = ctx
            .db
            .world_scene()
            .iter()
            .find(|scene| scene.name == scene_config.name)
            .ok_or("Scene does not exist")?;

        for position in scene_config.platform_positions {
            let platform = Platform::new(position.clone(), world_scene.scene_id);

            match ctx.db.platform().try_insert(platform) {
                Ok(inserted_platform) => {
                    log::info!(
                        "Platform registered at position ({}, {}) with id: {}",
                        inserted_platform.position.x,
                        inserted_platform.position.y,
                        inserted_platform.platform_id
                    );
                }
                Err(e) => {
                    log::error!("Error registering platform: {:?}", e);
                }
            }
        }

        Ok(())
    }

    fn initialize_green_slimes(
        ctx: &ReducerContext,
        scene_config: &WorldSceneConfig,
    ) -> Result<(), String> {
        let world_scene = ctx
            .db
            .world_scene()
            .iter()
            .find(|scene| scene.name == scene_config.name)
            .ok_or("Scene does not exist")?;

        for position in scene_config.green_slime_positions {
            let green_slime = GreenSlime::new(position.clone(), world_scene.scene_id);

            match ctx.db.green_slime().try_insert(green_slime) {
                Ok(inserted_green_slime) => {
                    log::info!(
                        "Green slime registered at position ({}, {}) with id: {}",
                        inserted_green_slime.position.x,
                        inserted_green_slime.position.y,
                        inserted_green_slime.green_slime_id
                    );
                }
                Err(e) => {
                    log::error!("Error registering green slime: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
