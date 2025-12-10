use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    BlockDevice,
    InputDevice,
    CharDevice,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub path: String,
    pub device_type: DeviceType,
    pub name: String,
    pub size_bytes: Option<u64>,
    pub mounted: bool,
    pub mount_point: Option<String>,
}
