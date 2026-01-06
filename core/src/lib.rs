use serde::{Deserialize, Serialize};

pub mod command;
pub mod device;
pub mod error;
pub mod input;

pub use command::{Command, CommandPayload, Response, ResponsePayload};
pub use device::{DeviceInfo, DeviceType};
pub use error::{LuCastraError, Result};
pub use input::{InputEvent, InputEventType, KeyCode};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemState {
    pub booted: bool,
    pub timestamp: u64,
}
