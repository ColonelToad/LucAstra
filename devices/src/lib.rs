use std::collections::HashMap;
use lucastra_core::{DeviceInfo, DeviceType, Result};
use tracing::info;

/// Device manager service: enumerates USB and input devices.
pub struct DeviceManager {
    devices: HashMap<String, DeviceInfo>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Scan for USB block devices and input devices (polling-based for MVP).
    pub fn scan(&mut self) -> Result<()> {
        info!("Scanning for devices...");

        // Mock USB device detection. In a real OS, enumerate /proc/scsi, lsblk, or libusb.
        let usb_device = DeviceInfo {
            path: "/dev/usb0".to_string(),
            device_type: DeviceType::BlockDevice,
            name: "USB Storage".to_string(),
            size_bytes: Some(1024 * 1024 * 1024), // 1GB mock
            mounted: false,
            mount_point: None,
        };
        self.devices.insert("/dev/usb0".to_string(), usb_device);

        // Mock keyboard device.
        let kbd_device = DeviceInfo {
            path: "/dev/input/kbd0".to_string(),
            device_type: DeviceType::InputDevice,
            name: "Keyboard".to_string(),
            size_bytes: None,
            mounted: false,
            mount_point: None,
        };
        self.devices.insert("/dev/input/kbd0".to_string(), kbd_device);

        // Mock mouse device.
        let mouse_device = DeviceInfo {
            path: "/dev/input/mouse0".to_string(),
            device_type: DeviceType::InputDevice,
            name: "Mouse".to_string(),
            size_bytes: None,
            mounted: false,
            mount_point: None,
        };
        self.devices.insert("/dev/input/mouse0".to_string(), mouse_device);

        info!("Device scan complete: {} devices found", self.devices.len());
        Ok(())
    }

    /// List all detected devices.
    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        Ok(self.devices.values().cloned().collect())
    }

    /// Get a device by path.
    pub fn get_device(&self, path: &str) -> Result<DeviceInfo> {
        self.devices
            .get(path)
            .cloned()
            .ok_or_else(|| lucastra_core::LuCastraError::DeviceNotFound(path.to_string()))
    }

    /// Mark a device as mounted.
    pub fn mount_device(&mut self, path: &str, mount_point: &str) -> Result<()> {
        if let Some(device) = self.devices.get_mut(path) {
            device.mounted = true;
            device.mount_point = Some(mount_point.to_string());
            info!("Device {} mounted at {}", path, mount_point);
            Ok(())
        } else {
            Err(lucastra_core::LuCastraError::DeviceNotFound(path.to_string()))
        }
    }

    /// Mark a device as unmounted.
    pub fn unmount_device(&mut self, path: &str) -> Result<()> {
        if let Some(device) = self.devices.get_mut(path) {
            device.mounted = false;
            device.mount_point = None;
            info!("Device {} unmounted", path);
            Ok(())
        } else {
            Err(lucastra_core::LuCastraError::DeviceNotFound(path.to_string()))
        }
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
