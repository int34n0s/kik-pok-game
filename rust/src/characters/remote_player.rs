use crate::handle_player_animation;

use godot::prelude::*;
use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};

/// Reduced strength to prevent jittering
const POSITION_CORRECTION_STRENGTH: f32 = 0.05;
/// Max distance before we snap instead of interpolate
const MAX_CORRECTION_DISTANCE: f32 = 100.0;
/// Don't correct small differences to reduce jittering
const CORRECTION_DEADBAND: f32 = 12.0;

/// Frames before vertical correction teleport
const VERTICAL_DIFF_FRAME_THRESHOLD: i32 = 10;
/// Frames before deadband teleport
const DEADBAND_FRAME_THRESHOLD: i32 = 30;
/// Minimum vertical difference to trigger correction
const VERTICAL_DIFF_THRESHOLD: f32 = 3.0;

#[derive(Clone, Debug)]
struct RemoteState {
    position: Vector2,
    direction: i32,
    is_jumping: bool,
}

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct RemotePlayerNode {
    speed: f32,
    jump_velocity: f32,

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

    animated_sprite: Option<Gd<AnimatedSprite2D>>,

    #[base]
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for RemotePlayerNode {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            speed: 100.0,
            jump_velocity: -300.0,
            last_server_state: None,
            current_direction: 0,
            current_jumping: false,
            was_jumping: false,
            vertical_diff_frame_count: 0,
            deadband_frame_count: 0,
            animated_sprite: None,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let mut velocity = self.base().get_velocity();
        let is_on_floor = self.base().is_on_floor();

        // Apply gravity when not on floor
        if !is_on_floor {
            velocity.y += self.base().get_gravity().y * delta as f32;
        }

        // Handle jump input - only jump when transitioning from not jumping to jumping
        let new_jump = self.current_jumping && !self.was_jumping && is_on_floor;
        if new_jump {
            velocity.y = self.jump_velocity;
        }

        // Update previous jump state for next frame
        self.was_jumping = self.current_jumping;

        // Handle horizontal movement from current inputs
        if self.current_direction != 0 {
            velocity.x = self.current_direction as f32 * self.speed;
        } else {
            velocity.x =
                godot::global::move_toward(velocity.x as f64, 0.0, self.speed as f64) as f32;
        }

        // Apply position correction if we have server data
        if let Some(server_state) = self.last_server_state.clone() {
            self.apply_position_correction(&server_state, delta);
        }

        // Update velocity and move using Godot's physics
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        // Handle animations
        if let Some(animated_sprite) = &mut self.animated_sprite {
            handle_player_animation(animated_sprite, self.current_direction as f32, is_on_floor);
        }
    }

    fn ready(&mut self) {
        if let Some(animated_sprite) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            self.animated_sprite = Some(animated_sprite);
        }
    }
}

#[godot_api]
impl RemotePlayerNode {
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
