use lucastra_core::{InputEvent, Result};

/// Input device driver abstraction (keyboard, mouse).
pub trait InputDriver {
    fn poll_event(&mut self) -> Result<Option<InputEvent>>;
    fn is_ready(&self) -> bool;
}

/// Mock input driver for testing.
pub struct MockInputDriver {
    event_queue: Vec<InputEvent>,
    index: usize,
}

impl MockInputDriver {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
            index: 0,
        }
    }

    pub fn inject_event(&mut self, event: InputEvent) {
        self.event_queue.push(event);
    }
}

impl Default for MockInputDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl InputDriver for MockInputDriver {
    fn poll_event(&mut self) -> Result<Option<InputEvent>> {
        if self.index < self.event_queue.len() {
            let event = self.event_queue[self.index].clone();
            self.index += 1;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    fn is_ready(&self) -> bool {
        self.index < self.event_queue.len()
    }
}
