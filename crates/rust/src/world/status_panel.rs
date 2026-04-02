use crate::DbConnection;

use godot::classes::{CenterContainer, ICenterContainer, InputEvent, InputEventKey, Label};
use godot::global::Key;
use godot::prelude::*;

use spacetimedb_sdk::DbContext;

#[derive(GodotClass)]
#[class(base=CenterContainer)]
pub struct StatusPanel {
    score_label: Option<Gd<Label>>,

    #[base]
    base: Base<CenterContainer>,
}

#[godot_api]
impl ICenterContainer for StatusPanel {
    fn init(base: Base<CenterContainer>) -> Self {
        Self {
            score_label: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.score_label = self.base().try_get_node_as::<Label>("ScoreLabel");
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(event) = event.try_cast::<InputEventKey>() {
            match event.get_keycode() {
                Key::TAB => {
                    if event.is_pressed() {
                        self.base_mut().set_visible(true);
                    }

                    if event.is_released() && !event.is_shift_pressed() {
                        self.base_mut().set_visible(false);
                    }
                }
                Key::ESCAPE => self.base_mut().set_visible(false),
                _ => {}
            }
        }
    }
}

#[godot_api]
impl StatusPanel {
    pub fn setup_multiplayer(connection: &DbConnection) {
        connection
            .subscription_builder()
            .subscribe(["SELECT * FROM world_scene", "SELECT * FROM player_score"]);
    }
}
