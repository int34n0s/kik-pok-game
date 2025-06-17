use spacetimedb::SpacetimeType;
use std::hash::{Hash, Hasher};

#[derive(SpacetimeType, Clone, Debug, Default)]
pub struct DbVector2 {
    pub x: f32,
    pub y: f32,
}

impl PartialEq for DbVector2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for DbVector2 {}

impl Hash for DbVector2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl DbVector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
