use crate::{ConnectionState, LevelManager, SpacetimeDBManager};

use godot::classes::{Button, IVBoxContainer, Label, LineEdit, VBoxContainer};
use godot::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum LoginUIState {
    #[default]
    Initial,
    LoginAttempted,
    Connecting,
    Connected,
    LoggedIn,
    Failed(String),
}

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct LoginScreen {
    ui_state: LoginUIState,
    level_manager: LevelManager,

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
            level_manager: LevelManager::new(),
            ui_state: LoginUIState::Initial,
            username_input: None,
            login_button: None,
            status_label: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        self.update_status_from_state();

        let Some(mut connection) = SpacetimeDBManager::get_write_connection() else {
            if self.ui_state == LoginUIState::LoginAttempted {
                self.update_status_with_failed_state("Could not get database connection!");
            }

            return;
        };

        if let Err(e) = connection.tick() {
            if self.ui_state == LoginUIState::LoginAttempted
                || self.ui_state == LoginUIState::Connecting
            {
                self.update_status_with_failed_state(&format!("Connection error: {:?}", e));
            }

            return;
        }

        let login_state = connection.login_module.get_state().clone();
        let should_check_and_login = connection.check_and_login();

        drop(connection);

        let new_ui_state = match login_state {
            ConnectionState::Disconnected if !matches!(self.ui_state, LoginUIState::Initial) => {
                Some(LoginUIState::Failed(
                    "Tried to connect to db, change username please".to_string(),
                ))
            }
            ConnectionState::Connected => Some(LoginUIState::Connected),
            ConnectionState::LoggedIn => Some(LoginUIState::LoggedIn),
            ConnectionState::LoginFailed(error)
                if !matches!(self.ui_state, LoginUIState::Failed(_)) =>
            {
                Some(LoginUIState::Failed(error))
            }
            _ => None,
        };

        if let Some(new_state) = new_ui_state {
            self.ui_state = new_state;
            self.update_status_from_state();
        }

        if should_check_and_login {
            self.transition_to_game();
        }
    }

    fn ready(&mut self) {
        self.setup_node_references();
        self.connect_signals();
        self.update_status_from_state();
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
        let Some(username_input) = &self.username_input else {
            godot_error!("Expected username_input");
            return;
        };

        let username = username_input.get_text().to_string();

        if username.trim().is_empty() {
            self.update_status_with_failed_state("Please enter a username");
            return;
        }

        self.ui_state = LoginUIState::LoginAttempted;

        let Some(mut connection) = SpacetimeDBManager::get_write_connection() else {
            self.update_status_with_failed_state("Could not get database connection!");
            return;
        };

        self.ui_state = LoginUIState::Connecting;

        match connection.connect(&username) {
            Ok(_) => {
                self.ui_state = LoginUIState::Connected;
            }
            Err(e) => {
                self.update_status_with_failed_state(&format!("Connection failed: {}", e));
                return;
            }
        }

        match connection.register_player(username, 1) {
            Ok(_) => {
                self.set_status("Registration request sent...");
            }
            Err(e) => {
                self.update_status_with_failed_state(&format!("Registration failed: {}", e));
                return;
            }
        }
    }

    fn update_status_from_state(&mut self) {
        let message = match &self.ui_state {
            LoginUIState::Initial => "Enter username and click login".to_string(),
            LoginUIState::LoginAttempted => "Initializing connection...".to_string(),
            LoginUIState::Connecting => "Connecting to database...".to_string(),
            LoginUIState::Connected => "Connected! Registering player...".to_string(),
            LoginUIState::LoggedIn => "Login successful! Entering game...".to_string(),
            LoginUIState::Failed(error) => error.clone(),
        };

        self.set_status(&message);
    }

    fn update_status_with_failed_state(&mut self, error: &str) {
        godot_print!("Error: {}", error);
        self.ui_state = LoginUIState::Failed(error.to_string());
        self.update_status_from_state();
    }

    fn set_status(&mut self, message: &str) {
        if let Some(status_label) = &mut self.status_label {
            status_label.set_text(message);
        }
    }

    fn transition_to_game(&mut self) {
        if let Some(mut scene_tree) = self.base().get_tree() {
            let error = scene_tree.change_scene_to_file(&self.level_manager.get_entry_scene_path());
            godot_print!("Transitioned to game. State: {:?}", error);
        }
    }
}
