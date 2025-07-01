use crate::SpacetimeDBManager;

use godot::prelude::*;
use godot::classes::{Area2D, CollisionShape2D, IArea2D, Timer};

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
        self.connect_body_entered_callback();

        let callback = self.base().callable("on_timer_timeout");
        if let Some(timer) = &mut self.timer {
            timer.connect("timeout", &callback);
        }
    }

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        self.delete_body_entered_callback();

        Self::set_deferred_collision_shape(&body, true);

        self.entered_body = Some(body);

        self.start_timer();
    }

    #[func]
    fn on_timer_timeout(&mut self) {
        let Some(entered_body) = &mut self.entered_body else {
            self.connect_body_entered_callback();

            return;
        };
        
        Self::set_deferred_collision_shape(entered_body, false);

        let Some(connection) = SpacetimeDBManager::get_read_connection() else {
            godot_error!("Could not get database connection!");
            return;
        };
        
        match connection.get_spawn_point() {
            Ok(Some(spawn_point)) => {
                entered_body.set_position(spawn_point);
            }
            Ok(None) => {
                godot_error!("Could not get spawn_point!");
            }
            Err(e) => {
                godot_error!("Could not get spawn_point: {:?}", e);
            }
        }

        self.entered_body = None;

        self.start_timer();
    }
    
    fn set_deferred_collision_shape(body: &Gd<Node2D>, state: bool) {
        if let Some(mut collision_shape) =
            body.try_get_node_as::<CollisionShape2D>("CollisionShape2D")
        {
            collision_shape.set_deferred("disabled", &state.to_variant());
        } else {
            godot_error!("Could not find CollisionShape2D on entered body");
        }
    }

    fn start_timer(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.start();
        } else {
            godot_error!("Timer not available to start");
        }
    }

    fn connect_body_entered_callback(&mut self) {
        let callback = self.base().callable("on_body_entered");
        self.base_mut().connect("body_entered", &callback);
    }

    fn delete_body_entered_callback(&mut self) {
        let callable = self.base().callable("on_body_entered");
        self.base_mut().disconnect("body_entered", &callable);
    }
}
