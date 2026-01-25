use crate::DbConnection;

use godot::classes::{CenterContainer, ICenterContainer, InputEvent, InputEventKey, Label};
use godot::global::Key;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=CenterContainer)]
pub struct SettingsPanel {
    score_label: Option<Gd<Label>>,

    #[base]
    base: Base<CenterContainer>,
}

#[godot_api]
impl ICenterContainer for SettingsPanel {
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
impl SettingsPanel {
    pub fn setup_multiplayer(_connection: &DbConnection) {}
}
