use crate::DbConnection;
use godot::classes::{AnimatedSprite2D, INode2D, RayCast2D};
use godot::prelude::*;
use spacetimedb_sdk::DbContext;

const SPEED: f32 = 60.0;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct GreenSlimeNode {
    direction: i32,

    ray_cast_right: Option<Gd<RayCast2D>>,
    ray_cast_left: Option<Gd<RayCast2D>>,

    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GreenSlimeNode {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            direction: 1,
            ray_cast_right: None,
            ray_cast_left: None,
            animated_sprite: None,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        if let Some(ref ray_cast_right) = self.ray_cast_right {
            if ray_cast_right.is_colliding() {
                self.direction = -1;

                if let Some(ref mut animated_sprite) = self.animated_sprite {
                    animated_sprite.set_flip_h(true);
                }
            }
        }

        if let Some(ref ray_cast_left) = self.ray_cast_left {
            if ray_cast_left.is_colliding() {
                self.direction = 1;

                if let Some(ref mut animated_sprite) = self.animated_sprite {
                    animated_sprite.set_flip_h(false);
                }
            }
        }

        // Update position
        let mut position = self.base().get_position();
        position.x += self.direction as f32 * SPEED * delta as f32;

        self.base_mut().set_position(position);
    }

    fn ready(&mut self) {
        self.ray_cast_right = self.base().try_get_node_as::<RayCast2D>("RayCastRight");
        self.ray_cast_left = self.base().try_get_node_as::<RayCast2D>("RayCastLeft");
        self.animated_sprite = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite");

        if self.ray_cast_right.is_none() {
            godot_error!("Could not find RayCastRight node");
        }
        if self.ray_cast_left.is_none() {
            godot_error!("Could not find RayCastLeft node");
        }
        if self.animated_sprite.is_none() {
            godot_error!("Could not find AnimatedSprite node");
        }
    }
}

impl GreenSlimeNode {
    pub fn setup_multiplayer(connection: &DbConnection) {
        connection
            .subscription_builder()
            .subscribe("SELECT * FROM green_slime");
    }
}
