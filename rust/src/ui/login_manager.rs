use crate::{CONNECTION_STATE, ConnectionState, Direction, GLOBAL_CONNECTION};
use std::sync::atomic::Ordering;

use godot::classes::{Button, IVBoxContainer, Label, LineEdit, VBoxContainer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct LoginScreen {
    username_input: Option<Gd<LineEdit>>,
    login_button: Option<Gd<Button>>,
    status_label: Option<Gd<Label>>,

    #[base]
    base: Base<VBoxContainer>,
}

#[godot_api]
impl IVBoxContainer for LoginScreen {
    fn init(base: Base<VBoxContainer>) -> Self {
        Self {
            username_input: None,
            login_button: None,
            status_label: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        let mut connection = GLOBAL_CONNECTION.lock().unwrap();

        if connection.is_connected() {
            if let Err(e) = connection.tick() {
                self.set_status(&format!("Connection error: {}", e));
            }
        }

        if CONNECTION_STATE.load(Ordering::SeqCst) != 0 {
            CONNECTION_STATE.store(0, Ordering::SeqCst);
            connection.state = ConnectionState::LoggedIn;
        }

        if connection.is_logged_in() {
            self.transition_to_game();
        }
    }

    fn ready(&mut self) {
        self.setup_node_references();
        self.connect_signals();
        self.update_ui_state();
    }
}

#[godot_api]
impl LoginScreen {
    fn setup_node_references(&mut self) {
        self.username_input = self.base().try_get_node_as::<LineEdit>("UsernameInput");
        self.login_button = self.base().try_get_node_as::<Button>("LoginButton");
        self.status_label = self.base().try_get_node_as::<Label>("StatusLabel");

        if self.username_input.is_none() {
            godot_error!("Could not find UsernameInput node");
        }
        if self.login_button.is_none() {
            godot_error!("Could not find LoginButton node");
        }
        if self.status_label.is_none() {
            godot_error!("Could not find StatusLabel node");
        }
    }

    fn connect_signals(&mut self) {
        let callback = self.base().callable("on_login_pressed");
        if let Some(login_button) = &mut self.login_button {
            login_button.connect("pressed", &callback);
        }
    }

    #[func]
    fn on_login_pressed(&mut self) {
        if let Some(username_input) = &self.username_input {
            let username = username_input.get_text().to_string();

            if username.trim().is_empty() {
                self.set_status("Please enter a username");
                return;
            }

            self.set_status("Connecting to server...");

            {
                let mut connection = GLOBAL_CONNECTION.lock().unwrap();
                match connection.connect("127.0.0.1:3000", "kik-pok", &username) {
                    Ok(_) => {
                        self.set_status("Connected! Registering player...");
                    }
                    Err(e) => {
                        self.set_status(&format!("Connection failed: {}", e));
                    }
                }
            }

            {
                let mut connection = GLOBAL_CONNECTION.lock().unwrap();
                match connection.register_player(username, 1, Direction::Right) {
                    Ok(_) => {
                        self.set_status("Registration request sent...");
                    }
                    Err(e) => {
                        self.set_status(&format!("Registration failed: {}", e));
                    }
                }
            }
        }
    }

    #[func]
    fn on_player_registered(&mut self, name: GString, scene_id: i32) {
        godot_print!(
            "Received player_registered signal: {} in scene {}",
            name,
            scene_id
        );

        // Update the connection state to LoggedIn
        {
            let mut connection = GLOBAL_CONNECTION.lock().unwrap();
            connection.set_state(ConnectionState::LoggedIn);
        }

        self.set_status("Registration successful! Loading game...");
    }

    fn set_status(&mut self, message: &str) {
        if let Some(status_label) = &mut self.status_label {
            status_label.set_text(message);
        }
    }

    fn update_ui_state(&mut self) {
        let state = GLOBAL_CONNECTION.lock().unwrap().get_connection_state();

        if let Some(login_button) = &mut self.login_button {
            login_button.set_disabled(state != ConnectionState::Disconnected);
        }

        if let Some(username_input) = &mut self.username_input {
            username_input.set_editable(state == ConnectionState::Disconnected);
        }
    }

    fn transition_to_game(&mut self) {
        if let Some(mut scene_tree) = self.base().get_tree() {
            let _result = scene_tree.change_scene_to_file("res://scenes/main.tscn");
        }
    }
}
