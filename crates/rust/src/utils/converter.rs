use godot::builtin::Vector2;

use crate::*;

impl From<Vector2> for DbVector2 {
    fn from(value: Vector2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<DbVector2> for Vector2 {
    fn from(value: DbVector2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
