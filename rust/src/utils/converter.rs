use godot::builtin::Vector2;

use crate::player::PlayerState;
use crate::multiplayer::spacetimedb_client::{Positioning, Direction, DbVector2};

impl PlayerState {
    /// Convert PlayerState to Positioning for SpacetimeDB
    pub fn to_positioning(&self) -> Positioning {
        let direction = if self.direction > 0.0 {
            Direction::Right
        } else if self.direction < 0.0 {
            Direction::Left
        } else {
            Direction::None
        };

        Positioning {
            coordinates: DbVector2 {
                x: self.position.x,
                y: self.position.y,
            },
            direction,
            in_on_floor: self.is_on_floor,
        }
    }

    /// Create PlayerState from Positioning
    pub fn from_positioning(positioning: &Positioning) -> Self {
        let direction = match positioning.direction {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            Direction::None => 0.0,
        };

        Self {
            position: Vector2::new(positioning.coordinates.x, positioning.coordinates.y),
            direction,
            is_on_floor: positioning.in_on_floor,
        }
    }
}

impl Positioning {
    /// Convert Positioning to PlayerState
    pub fn to_player_state(&self) -> PlayerState {
        PlayerState::from_positioning(self)
    }

    /// Create Positioning from PlayerState
    pub fn from_player_state(state: &PlayerState) -> Self {
        state.to_positioning()
    }
}
