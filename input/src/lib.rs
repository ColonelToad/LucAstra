use lucastra_core::{InputEvent, Result};
use lucastra_hal::InputDriver;
use std::collections::VecDeque;
use tracing::info;

/// Input manager service: polls input devices and buffers events.
pub struct InputManager {
    drivers: Vec<Box<dyn InputDriver + Send>>,
    event_queue: VecDeque<InputEvent>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            drivers: Vec::new(),
            event_queue: VecDeque::new(),
        }
    }

    /// Register an input driver (keyboard, mouse, etc.).
    pub fn register_driver(&mut self, driver: Box<dyn InputDriver + Send>) {
        info!("Registering input driver");
        self.drivers.push(driver);
    }

    /// Poll all registered input drivers for events.
    pub fn poll_events(&mut self) -> Result<()> {
        for driver in &mut self.drivers {
            while let Some(event) = driver.poll_event()? {
                self.event_queue.push_back(event);
            }
        }
        Ok(())
    }

    /// Retrieve the next buffered input event (non-blocking).
    pub fn get_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }

    /// Check if there are pending events.
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Clear all pending events.
    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
