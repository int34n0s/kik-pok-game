use godot::classes::{Button, IVBoxContainer, StyleBoxFlat, VBoxContainer};
use godot::prelude::*;

/// Room color configuration that can be set from Rust/SpacetimeDB
#[derive(Clone, Debug, Default)]
pub struct RoomColor {
    pub background: Color,
    pub border: Color,
    pub background_hover: Color,
    pub border_hover: Color,
}

impl RoomColor {
    pub fn green() -> Self {
        Self {
            background: Color::from_rgb(0.15, 0.25, 0.15),
            border: Color::from_rgb(0.3, 0.7, 0.3),
            background_hover: Color::from_rgb(0.20, 0.32, 0.20),
            border_hover: Color::from_rgb(0.4, 0.85, 0.4),
        }
    }

    pub fn blue() -> Self {
        Self {
            background: Color::from_rgb(0.15, 0.15, 0.25),
            border: Color::from_rgb(0.3, 0.5, 0.9),
            background_hover: Color::from_rgb(0.20, 0.20, 0.32),
            border_hover: Color::from_rgb(0.4, 0.6, 1.0),
        }
    }

    pub fn orange() -> Self {
        Self {
            background: Color::from_rgb(0.25, 0.18, 0.1),
            border: Color::from_rgb(0.9, 0.5, 0.2),
            background_hover: Color::from_rgb(0.32, 0.24, 0.15),
            border_hover: Color::from_rgb(1.0, 0.6, 0.3),
        }
    }

    pub fn red() -> Self {
        Self {
            background: Color::from_rgb(0.25, 0.12, 0.12),
            border: Color::from_rgb(0.9, 0.3, 0.3),
            background_hover: Color::from_rgb(0.32, 0.18, 0.18),
            border_hover: Color::from_rgb(1.0, 0.4, 0.4),
        }
    }

    pub fn purple() -> Self {
        Self {
            background: Color::from_rgb(0.2, 0.12, 0.25),
            border: Color::from_rgb(0.7, 0.3, 0.9),
            background_hover: Color::from_rgb(0.28, 0.18, 0.32),
            border_hover: Color::from_rgb(0.85, 0.4, 1.0),
        }
    }

    pub fn gray() -> Self {
        Self {
            background: Color::from_rgb(0.272, 0.272, 0.272),
            border: Color::from_rgb(0.4, 0.4, 0.4),
            background_hover: Color::from_rgb(0.35, 0.35, 0.35),
            border_hover: Color::from_rgb(0.5, 0.5, 0.5),
        }
    }
}

#[derive(GodotClass)]
#[class(base=VBoxContainer)]
pub struct RoomSelectScreen {
    room_buttons: Vec<Option<Gd<Button>>>,
    room_colors: Vec<RoomColor>,

    #[base]
    base: Base<VBoxContainer>,
}

#[godot_api]
impl IVBoxContainer for RoomSelectScreen {
    fn init(base: Base<VBoxContainer>) -> Self {
        Self {
            room_buttons: Vec::new(),
            room_colors: Vec::new(),
            base,
        }
    }

    fn ready(&mut self) {
        self.setup_room_buttons();
        self.apply_colors();
        self.connect_signals();
    }
}

#[godot_api]
impl RoomSelectScreen {
    fn setup_room_buttons(&mut self) {
        let grid = self.base().try_get_node_as::<Node>("GridContainer");

        if grid.is_none() {
            godot_error!("Could not find GridContainer node");
            return;
        }

        // TODO: this depends on the server state
        self.room_buttons = (1..=6)
            .map(|i| {
                let path = format!("GridContainer/Room{i}Button");
                self.base().try_get_node_as::<Button>(&path)
            })
            .collect();

        for (i, btn) in self.room_buttons.iter().enumerate() {
            if btn.is_none() {
                godot_error!("Could not find Room{}Button", i + 1);
            }
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn connect_signals(&mut self) {
        let callable_base = self.base().callable("on_room_pressed");
        for i in 0..self.room_buttons.len() {
            if let Some(btn) = &mut self.room_buttons[i] {
                let room_num = (i + 1) as i32;
                let callback = callable_base.bindv(&varray![room_num]);
                btn.connect("pressed", &callback);
            }
        }
    }

    fn apply_colors(&mut self) {
        for (i, button) in self.room_buttons.iter_mut().enumerate() {
            if let Some(btn) = button {
                let color = self
                    .room_colors
                    .get(i)
                    .cloned()
                    .unwrap_or_else(RoomColor::gray);
                Self::apply_button_style(btn, &color);
            }
        }
    }

    fn apply_button_style(button: &mut Gd<Button>, color: &RoomColor) {
        // Create normal style
        let mut normal_style = StyleBoxFlat::new_gd();
        normal_style.set_bg_color(color.background);
        normal_style.set_border_width_all(3);
        normal_style.set_border_color(color.border);
        normal_style.set_content_margin_all(10.0);

        // Create hover style
        let mut hover_style = StyleBoxFlat::new_gd();
        hover_style.set_bg_color(color.background_hover);
        hover_style.set_border_width_all(3);
        hover_style.set_border_color(color.border_hover);
        hover_style.set_content_margin_all(10.0);

        // Apply styles
        button.add_theme_stylebox_override("normal", &normal_style);
        button.add_theme_stylebox_override("hover", &hover_style);
        button.add_theme_stylebox_override("pressed", &normal_style);
    }

    /// Set the color for a specific room (0-indexed)
    #[func]
    #[allow(clippy::needless_pass_by_value, clippy::cast_sign_loss)]
    pub fn set_room_color(&mut self, room_index: i32, color_name: GString) {
        let idx = room_index as usize;
        if idx >= self.room_colors.len() {
            godot_error!("Invalid room index: {room_index}");
            return;
        }

        let color = match color_name.to_string().to_lowercase().as_str() {
            "green" => RoomColor::green(),
            "blue" => RoomColor::blue(),
            "orange" => RoomColor::orange(),
            "red" => RoomColor::red(),
            "purple" => RoomColor::purple(),
            _ => RoomColor::gray(),
        };

        self.room_colors[idx] = color.clone();

        // Apply immediately if button exists
        if let Some(Some(btn)) = self.room_buttons.get_mut(idx) {
            Self::apply_button_style(btn, &color);
        }
    }

    #[func]
    fn on_room_pressed(&self, room_number: i32) {
        godot_print!("Room {room_number} selected. {}", self.base.to_string());
        // TODO: Connect to SpacetimeDB to join room
    }
}
