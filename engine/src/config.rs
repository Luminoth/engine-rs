use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}
