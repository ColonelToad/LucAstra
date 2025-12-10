use serde::{Deserialize, Serialize};

pub mod command;
pub mod device;
pub mod error;
pub mod input;

pub use command::{Command, CommandPayload, Response, ResponsePayload};
pub use device::{DeviceInfo, DeviceType};
pub use error::{LuCastraError, Result};
pub use input::{InputEvent, InputEventType, KeyCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub booted: bool,
    pub timestamp: u64,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            booted: false,
            timestamp: 0,
        }
    }
}
