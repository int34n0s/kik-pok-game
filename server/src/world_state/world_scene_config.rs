use crate::elements::coin::{coin, Coin};
use crate::elements::world_scene::{world_scene, WorldScene};
use crate::elements::DbVector2;

use itertools::Itertools;
use spacetimedb::{ReducerContext, Table};

pub struct WorldSceneConfig {
    name: &'static str,
    spawn_point: DbVector2,
    coins: &'static [CoinSetup],
}

pub struct CoinSetup {
    position: DbVector2,
}

impl WorldSceneConfig {
    const SCENES: &'static [WorldSceneConfig] = &[WorldSceneConfig {
        name: "Main",
        spawn_point: DbVector2 { x: -15.0, y: -35.0 },
        coins: &[
            CoinSetup {
                position: DbVector2 { x: 80.0, y: -25.0 },
            },
            CoinSetup {
                position: DbVector2 { x: 98.0, y: -25.0 },
            },
            CoinSetup {
                position: DbVector2 { x: 178.0, y: -25.0 },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 178.0,
                    y: -120.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 498.0,
                    y: -104.0,
                },
            },
            CoinSetup {
                position: DbVector2 { x: 530.0, y: -88.0 },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 690.0,
                    y: -104.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 626.0,
                    y: -344.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 642.0,
                    y: -328.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 674.0,
                    y: -312.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 834.0,
                    y: -312.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 882.0,
                    y: -312.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 754.0,
                    y: -296.0,
                },
            },
            CoinSetup {
                position: DbVector2 {
                    x: 784.0,
                    y: -296.0,
                },
            },
            CoinSetup {
                position: DbVector2 { x: 834.0, y: 23.0 },
            },
        ],
    }];

    pub fn initialize_all_scenes(ctx: &ReducerContext) -> Result<(), String> {
        for scene_config in Self::SCENES {
            ctx.db.world_scene().insert(WorldScene::new(
                scene_config.name.to_string(),
                scene_config.spawn_point.clone(),
                ctx.timestamp,
            ));

            let world_scene = ctx
                .db
                .world_scene()
                .iter()
                .find(|scene| scene.name == scene_config.name)
                .ok_or("Scene does not exist")?;

            Self::initialize_coins(ctx, scene_config, &world_scene)?;

            log::info!("Initialized scene: {}", scene_config.name);
        }

        Ok(())
    }

    fn initialize_coins(
        ctx: &ReducerContext,
        scene_config: &WorldSceneConfig,
        world_scene: &WorldScene,
    ) -> Result<(), String> {
        let coin_unique_positions = scene_config
            .coins
            .iter()
            .map(|setup| setup.position.clone())
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
                    log::error!("Error registering coin: {e:?}");
                }
            }
        }

        Ok(())
    }
}
