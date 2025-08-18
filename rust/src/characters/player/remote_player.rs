use crate::{DbPlayer, MultiplayerManager, RustLibError};

use super::BasicPlayer;

use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Label, ResourceLoader};
use godot::obj::BaseMut;
use godot::prelude::*;

/// Reduced strength to prevent jittering
const POSITION_CORRECTION_STRENGTH: f32 = 0.05;
/// Max distance before we snap instead of interpolate
const MAX_CORRECTION_DISTANCE: f32 = 100.0;
/// Don't correct small differences to reduce jittering
const CORRECTION_DEADBAND: f32 = 1.0;

/// Frames before vertical correction teleport
const VERTICAL_DIFF_FRAME_THRESHOLD: i32 = 10;
/// Frames before deadband teleport
const DEADBAND_FRAME_THRESHOLD: i32 = 60;
/// Minimum vertical difference to trigger correction
const VERTICAL_DIFF_THRESHOLD: f32 = 3.0;

pub const PLAYER_SCENE_PATH: &str = "res://scenes/characters/remote_player.tscn";

#[derive(Clone, Debug)]
struct RemoteState {
    position: Vector2,
    direction: i32,
    is_jumping: bool,
}

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct RemotePlayerNode {
    basic_player: BasicPlayer,

    /// Server state
    last_server_state: Option<RemoteState>,

    /// Current input for responsive movement
    current_direction: i32,
    current_jumping: bool,

    /// Track previous jump state to detect new jumps
    was_jumping: bool,

    /// Count frames for vertical difference condition
    vertical_diff_frame_count: i32,
    /// Count frames for deadband condition
    deadband_frame_count: i32,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for RemotePlayerNode {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            basic_player: BasicPlayer::new(),
            last_server_state: None,
            current_direction: 0,
            current_jumping: false,
            was_jumping: false,
            vertical_diff_frame_count: 0,
            deadband_frame_count: 0,
            base,
        }
    }

    fn ready(&mut self) {
        if let Some(animated_sprite) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            self.basic_player.animated_sprite = Some(animated_sprite);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let mut velocity = self.base().get_velocity();
        let is_on_floor = self.base().is_on_floor();

        if !is_on_floor {
            velocity.y += self.base().get_gravity().y * delta as f32;
        }

        // Handle jump input using basic player - only jump when transitioning from not jumping to jumping
        let new_jump = self.current_jumping && !self.was_jumping && is_on_floor;
        if new_jump {
            self.basic_player.handle_jump(&mut velocity);
        }

        // Update previous jump state for next frame
        self.was_jumping = self.current_jumping;

        self.basic_player
            .apply_horizontal_movement(&mut velocity, self.current_direction as f32);

        if let Some(server_state) = self.last_server_state.clone() {
            self.apply_position_correction(&server_state, delta);
        }

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        self.basic_player
            .handle_animation(self.current_direction as f32, is_on_floor);
    }
}

#[godot_api]
impl RemotePlayerNode {
    pub fn spawn_object(
        mut base: BaseMut<MultiplayerManager>,
        player: &DbPlayer,
    ) -> Result<Gd<RemotePlayerNode>, RustLibError> {
        let player_id = player.identity;
        let name = &player.name;
        let position = Vector2::new(player.state.position.x, player.state.position.y);

        godot_print!(
            "Spawning remote player {} ({}) at ({}, {})",
            player_id,
            name,
            position.x,
            position.y
        );

        let mut resource_loader = ResourceLoader::singleton();
        let Some(packed_scene) = resource_loader.load(PLAYER_SCENE_PATH) else {
            godot_print!("Failed to load resource at {}", PLAYER_SCENE_PATH);
            return Err(RustLibError::ResourceLoadError(
                PLAYER_SCENE_PATH.to_string(),
            ));
        };

        let Ok(scene) = packed_scene.try_cast::<PackedScene>() else {
            godot_print!("Failed to cast resource to PackedScene");
            return Err(RustLibError::ResourceCastError(
                PLAYER_SCENE_PATH.to_string(),
                "PackedScene".to_string(),
            ));
        };

        let Some(instance) = scene.instantiate() else {
            godot_print!("Failed to instantiate scene");
            return Err(RustLibError::ResourceInstantiateError(
                PLAYER_SCENE_PATH.to_string(),
            ));
        };

        let Ok(mut remote_player) = instance.try_cast::<RemotePlayerNode>() else {
            godot_print!("Failed to cast instance to Player");
            return Err(RustLibError::ResourceCastError(
                PLAYER_SCENE_PATH.to_string(),
                "RemotePlayerNode".to_string(),
            ));
        };

        remote_player.set_position(position);
        remote_player.set_name(&GString::from(player_id.to_string()));

        if let Some(mut player_name) = remote_player.try_get_node_as::<Label>("PlayerName") {
            player_name.set_text(&GString::from(name));
        }

        base.add_child(&remote_player);

        Ok(remote_player)
    }

    #[func]
    pub fn get_player_state(&self) -> Vector2 {
        self.base().get_global_position()
    }

    #[func]
    pub fn set_player_position(&mut self, direction: i32, is_jumping: bool, position: Vector2) {
        self.last_server_state = Some(RemoteState {
            position,
            direction,
            is_jumping,
        });

        self.current_direction = direction;
        self.current_jumping = is_jumping;
    }

    fn apply_position_correction(&mut self, server_state: &RemoteState, _delta: f64) {
        let current_pos = self.base().get_global_position();
        let server_pos = server_state.position;
        let distance = current_pos.distance_to(server_pos);

        // If we're too far off, snap to server position
        if distance > MAX_CORRECTION_DISTANCE {
            godot_print!("Corrected position with distance: {:?}", server_pos);
            self.base_mut().set_global_position(server_pos);
            return;
        }

        // Check for large vertical discrepancies (platform issues)
        let vertical_diff = (server_pos.y - current_pos.y).abs();
        let vertical_condition = self.base().is_on_floor()
            && server_state.direction != 0
            && !server_state.is_jumping
            && vertical_diff > VERTICAL_DIFF_THRESHOLD;

        if vertical_condition {
            self.vertical_diff_frame_count += 1;

            if self.vertical_diff_frame_count > VERTICAL_DIFF_FRAME_THRESHOLD {
                self.base_mut().set_global_position(server_pos);

                self.vertical_diff_frame_count = 0;

                return;
            }

            return;
        } else {
            self.vertical_diff_frame_count = 0;
        }

        if distance <= CORRECTION_DEADBAND {
            self.deadband_frame_count = 0;
            return;
        }

        if self.deadband_frame_count > DEADBAND_FRAME_THRESHOLD {
            self.base_mut().set_global_position(server_pos);

            return;
        }

        let correction_vector = (server_pos - current_pos) * POSITION_CORRECTION_STRENGTH;
        let corrected_pos = current_pos + correction_vector;
        self.base_mut().set_global_position(corrected_pos);

        self.deadband_frame_count += 1;
    }
}
