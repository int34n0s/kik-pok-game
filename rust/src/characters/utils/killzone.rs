use crate::GLOBAL_CONNECTION;

use godot::classes::{Area2D, CollisionShape2D, Engine, IArea2D, Timer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct KillZoneArea {
    timer: Option<Gd<Timer>>,

    entered_body: Option<Gd<Node2D>>,

    #[base]
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for KillZoneArea {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            timer: None,
            entered_body: None,
        }
    }

    fn ready(&mut self) {
        self.timer = self.base().try_get_node_as::<Timer>("Timer");
        if self.timer.is_none() {
            godot_error!("Could not find Timer node");
        }

        self.connect_signals();
    }
}

#[godot_api]
impl KillZoneArea {
    fn connect_signals(&mut self) {
        let callback = self.base().callable("on_body_entered");
        self.base_mut().connect("body_entered", &callback);

        let callback = self.base().callable("on_timer_timeout");
        if let Some(timer) = &mut self.timer {
            timer.connect("timeout", &callback);
        }
    }

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        let callable = self.base().callable("on_body_entered");
        self.base_mut().disconnect("body_entered", &callable);

        let mut engine = Engine::singleton();
        engine.set_time_scale(0.35);

        if let Some(mut collision_shape) =
            body.try_get_node_as::<CollisionShape2D>("CollisionShape2D")
        {
            collision_shape.set_deferred("disabled", &true.to_variant());
        } else {
            godot_error!("Could not find CollisionShape2D on entered body");
        }

        self.entered_body = Some(body);

        if let Some(timer) = &mut self.timer {
            timer.start();
        } else {
            godot_error!("Timer not available to start");
        }
    }

    #[func]
    fn on_timer_timeout(&mut self) {
        let Some(entered_body) = &mut self.entered_body else {
            let callback = self.base().callable("on_body_entered");
            self.base_mut().connect("body_entered", &callback);

            return;
        };

        let mut engine = Engine::singleton();
        engine.set_time_scale(1.0);

        let connection = GLOBAL_CONNECTION.lock().unwrap();
        if !connection.is_connected() {
            return;
        }

        if let Some(mut collision_shape) =
            entered_body.try_get_node_as::<CollisionShape2D>("CollisionShape2D")
        {
            collision_shape.set_deferred("disabled", &false.to_variant());
        } else {
            godot_error!("Could not find CollisionShape2D on entered body");
        }

        if let Some(spawn_point) = connection.get_spawn_point() {
            entered_body.set_position(spawn_point);
        } else {
            godot_error!("Could not get spawn_point!");
        }

        self.entered_body = None;

        if let Some(timer) = &mut self.timer {
            timer.start();
        } else {
            godot_error!("Timer not available to start");
        }
    }
}
