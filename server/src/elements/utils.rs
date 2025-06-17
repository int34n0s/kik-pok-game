use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Debug, Default)]
pub struct DbVector2 {
    pub x: f32,
    pub y: f32,
}
