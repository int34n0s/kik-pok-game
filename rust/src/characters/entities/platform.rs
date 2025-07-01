use godot::classes::{AnimatableBody2D, IAnimatableBody2D};
use godot::prelude::*;
use spacetimedb_sdk::DbContext;
use crate::DbConnection;

#[derive(GodotClass)]
#[class(base=AnimatableBody2D)]
pub struct PlatformNode {
    #[base]
    base: Base<AnimatableBody2D>,
}

#[godot_api]
impl IAnimatableBody2D for PlatformNode {
    fn init(base: Base<AnimatableBody2D>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {}
}

impl PlatformNode {
    pub fn setup_multiplayer(connection: &DbConnection) {
        connection
            .subscription_builder()
            .subscribe("SELECT * FROM platform");
    }
}