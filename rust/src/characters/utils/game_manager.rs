use crate::{DbConnection, DbVector2};

use godot::classes::{INode, Label, Node};
use godot::prelude::*;

use spacetimedb_sdk::DbContext;

use std::sync::RwLock;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref COIN_QUEUE: RwLock<Vec<DbVector2>> = RwLock::new(Vec::new());
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameManager {
    score_label: Option<Gd<Label>>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for GameManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            score_label: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.score_label = self.base().try_get_node_as::<Label>("ScoreLabel");
    }

    fn process(&mut self, _delta: f64) {}
}

#[godot_api]
impl GameManager {
    pub fn setup_multiplayer(connection: &DbConnection) {
        connection
            .subscription_builder()
            .subscribe(["SELECT * FROM world_scene", "SELECT * FROM player_score"]);
    }
}
