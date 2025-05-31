use crate::{ConnectionState, Direction, GLOBAL_CONNECTION, initialize_connection};

use godot::classes::{Button, IVBoxContainer, Label, LineEdit, VBoxContainer};
use godot::global::{godot_error, godot_print};
use godot::obj::{Base, Gd, WithBaseField};
use godot::register::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct LoginScreen {
    // host_input: Option<Gd<LineEdit>>,
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
            // host_input: None,
            username_input: None,
            login_button: None,
            status_label: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        let mut connection = GLOBAL_CONNECTION.lock().unwrap();
        if !connection.is_connected() {
            return;
        }

        if let Err(e) = connection.tick() {
            self.set_status(&format!("Connection error: {}", e));
        }

        if connection.check_and_login() {
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
        // self.host_input = self.base().try_get_node_as::<LineEdit>("HostInput");
        self.username_input = self.base().try_get_node_as::<LineEdit>("UsernameInput");
        self.login_button = self.base().try_get_node_as::<Button>("LoginButton");
        self.status_label = self.base().try_get_node_as::<Label>("StatusLabel");

        // if self.host_input.is_none() {
        //     godot_error!("Could not find HostInput node");
        // }
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
        // let Some(host_input) = &self.host_input else {
        //     godot_error!("Expected host_input");
        //     return;
        // };

        let Some(username_input) = &self.username_input else {
            godot_error!("Expected username_input");

            return;
        };

        // let host = host_input.get_text().to_string();
        let username = username_input.get_text().to_string();
        // 
        // if host.trim().is_empty() {
        //     self.set_status("Please enter a host address");
        //     return;
        // }

        if username.trim().is_empty() {
            self.set_status("Please enter a username");
            return;
        }

        self.set_status("Initializing connection...");
        initialize_connection("kik-pok");

        let mut connection = GLOBAL_CONNECTION.lock().unwrap();
        if !connection.is_connected() {
            self.set_status("Connection not available");
            return;
        }

        if connection.is_player_logged_in(&username) {
            self.set_status("A player with this username is already logged in");
            return;
        }
        
        godot_print!("Reconnecting...");

        match connection.connect(&username) {
            Ok(_) => {
                self.set_status("Connected! Registering player...");
            }
            Err(e) => {
                self.set_status(&format!("Connection failed: {}", e));
                return;
            }
        }

        match connection.register_player(username, 1, Direction::Right) {
            Ok(_) => {
                self.set_status("Registration request sent...");
            }
            Err(e) => {
                self.set_status(&format!("Registration failed: {}", e));
            }
        }
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

        // if let Some(host_input) = &mut self.host_input {
        //     host_input.set_editable(state == ConnectionState::Disconnected);
        // }
    }

    fn transition_to_game(&mut self) {
        if let Some(mut scene_tree) = self.base().get_tree() {
            let _result = scene_tree.change_scene_to_file("res://scenes/main.tscn");
        }
    }
}
