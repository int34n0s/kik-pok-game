pub struct LevelManager {}

impl LevelManager {
    pub fn new() -> Self {
        LevelManager {}
    }
    
    pub fn get_entry_scene_path(&self) -> String {
        "res://scenes/world/entry.tscn".to_string()
    }
}