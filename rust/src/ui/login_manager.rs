use crate::{ConnectionState, SpacetimeDBManager};

use godot::prelude::*;
use godot::global::HorizontalAlignment;
use godot::classes::{Button, Control, IControl, Label, LineEdit, VBoxContainer};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct LoginScreen {
    db_manager: SpacetimeDBManager,

    username_input: Option<Gd<LineEdit>>,
    connect_button: Option<Gd<Button>>,
    login_button: Option<Gd<Button>>,
    status_label: Option<Gd<Label>>,

    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for LoginScreen {
    fn init(base: Base<Control>) -> Self {
        Self {
            db_manager: SpacetimeDBManager::new(),
            username_input: None,
            connect_button: None,
            login_button: None,
            status_label: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.setup_ui();
        self.update_ui_state();
    }

    fn process(&mut self, _delta: f64) {
        // Handle connection ticking
        if self.db_manager.is_connected() {
            if let Err(e) = self.db_manager.tick() {
                self.set_status(&format!("Connection error: {}", e));
            }
        }

        // Check if we're logged in and should transition to game
        if self.db_manager.is_logged_in() {
            self.transition_to_game();
        }
    }
}

#[godot_api]
impl LoginScreen {
    fn setup_ui(&mut self) {
        // Create main container
        let mut main_container = VBoxContainer::new_alloc();
        main_container.set_anchor_and_offset(Side::LEFT, 0.0, 50.0);
        main_container.set_anchor_and_offset(Side::TOP, 0.0, 50.0);
        main_container.set_anchor_and_offset(Side::RIGHT, 1.0, -50.0);
        main_container.set_anchor_and_offset(Side::BOTTOM, 1.0, -50.0);

        // Title label
        let mut title_label = Label::new_alloc();
        title_label.set_text("Kik-Pok Game");
        title_label.set_horizontal_alignment(HorizontalAlignment::CENTER);
        main_container.add_child(&title_label);

        // Status label
        let mut status_label = Label::new_alloc();
        status_label.set_text("Ready to connect");
        status_label.set_horizontal_alignment(HorizontalAlignment::CENTER);
        main_container.add_child(&status_label);
        self.status_label = Some(status_label);

        // Connect button
        let mut connect_button = Button::new_alloc();
        connect_button.set_text("Connect to Server");
        connect_button.connect("pressed", &self.base().callable("on_connect_pressed"));
        main_container.add_child(&connect_button);
        self.connect_button = Some(connect_button);

        // Username input
        let mut username_input = LineEdit::new_alloc();
        username_input.set_placeholder("Enter your username");
        username_input.set_max_length(20);
        main_container.add_child(&username_input);
        self.username_input = Some(username_input);

        // Login button
        let mut login_button = Button::new_alloc();
        login_button.set_text("Login");
        login_button.connect("pressed", &self.base().callable("on_login_pressed"));
        main_container.add_child(&login_button);
        self.login_button = Some(login_button);

        self.base_mut().add_child(&main_container);
    }

    #[func]
    fn on_connect_pressed(&mut self) {
        self.set_status("Connecting to server...");

        match self.db_manager.connect("127.0.0.1:3000", "kik-pok") {
            Ok(_) => {
                self.set_status("Connected! Enter your username.");
                self.update_ui_state();
            }
            Err(e) => {
                self.set_status(&format!("Connection failed: {}", e));
                self.update_ui_state();
            }
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

            self.set_status("Registering player...");

            match self.db_manager.register_player(username) {
                Ok(_) => {
                    self.set_status("Login successful! Loading game...");
                }
                Err(e) => {
                    self.set_status(&format!("Login failed: {}", e));
                }
            }
        }
    }

    fn set_status(&mut self, message: &str) {
        if let Some(status_label) = &mut self.status_label {
            status_label.set_text(message);
        }
        godot_print!("Login Screen: {}", message);
    }

    fn update_ui_state(&mut self) {
        let state = self.db_manager.get_connection_state();

        if let Some(connect_button) = &mut self.connect_button {
            connect_button.set_disabled(state != ConnectionState::Disconnected);
        }

        if let Some(login_button) = &mut self.login_button {
            login_button.set_disabled(state != ConnectionState::Connected);
        }

        if let Some(username_input) = &mut self.username_input {
            username_input.set_editable(state == ConnectionState::Connected);
        }
    }

    fn transition_to_game(&mut self) {
        godot_print!("Transitioning to main game...");

        // Signal to the scene tree that we should switch to the main game scene
        if let Some(mut scene_tree) = self.base().get_tree() {
            let _result = scene_tree.change_scene_to_file("res://scenes/main.tscn");
        }
    }
}
