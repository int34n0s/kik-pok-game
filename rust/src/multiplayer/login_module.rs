use crate::RustLibError;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
    LoggedIn,
    LoginFailed(String),
}

#[derive(Default)]
pub struct LoginModule {
    state: ConnectionState,
    scene_id: Option<u32>,
    player_name: Option<String>,
}

impl LoginModule {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            scene_id: None,
            player_name: None,
        }
    }

    pub fn set_player_name(&mut self, player_name: String) {
        self.player_name = Some(player_name);
    }

    pub fn set_scene_id(&mut self, scene_id: u32) {
        self.scene_id = Some(scene_id);
    }
    
    pub fn get_state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn get_state_mut(&mut self) -> &mut ConnectionState {
        &mut self.state
    }

    pub fn get_scene_id(&self) -> Option<u32> {
        self.scene_id
    }

    pub fn get_player_name(&self) -> Option<&str> {
        self.player_name.as_deref()
    }
    
    pub fn require_logged_in(&self) -> Result<(), RustLibError> {
        if self.state == ConnectionState::LoggedIn {
            return Ok(());
        }
        
        Err(RustLibError::WrongConnectionState("Unexpected function call on non-logged in state".to_string()))
    }
}