use crate::DbConnection;
use godot::classes::{AnimatableBody2D, AnimationPlayer, IAnimatableBody2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=AnimatableBody2D)]
pub struct PlatformNode {
    animation_player: Option<Gd<AnimationPlayer>>,

    #[base]
    base: Base<AnimatableBody2D>,
}

#[godot_api]
impl IAnimatableBody2D for PlatformNode {
    fn init(base: Base<AnimatableBody2D>) -> Self {
        Self {
            base,
            animation_player: None,
        }
    }

    fn ready(&mut self) {
        self.animation_player = self
            .base()
            .try_get_node_as::<AnimationPlayer>("AnimationPlayer");
    }
}

#[godot_api]
impl PlatformNode {
    /// Synchronize platform animation based on a shared world time (in microseconds).
    /// Uses AnimationPlayer.advance(time) so Godot handles looping/ping-pong correctly.
    #[func]
    pub fn sync_based_on_time(&mut self, time_microseconds: f64) {
        let Some(ref mut animation_player) = self.animation_player else {
            return;
        };

        // Start (or restart) the current animation at t=0, then advance by the shared time.
        let time_seconds = time_microseconds / 1_000_000.0;
        animation_player.call("play", &["move".to_variant()]);
        animation_player.call("advance", &[time_seconds.to_variant()]);
    }
}

impl PlatformNode {
    pub fn setup_multiplayer(_connection: &DbConnection) {}
}
