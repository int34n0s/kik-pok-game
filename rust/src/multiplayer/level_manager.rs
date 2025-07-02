pub struct LevelManager {}

impl Default for LevelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelManager {
    pub fn new() -> Self {
        LevelManager {}
    }

    pub fn get_entry_scene_path(&self) -> String {
        "res://scenes/world/entry.tscn".to_string()
    }
}
