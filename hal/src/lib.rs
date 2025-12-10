pub mod block;
pub mod filesystem;
pub mod input;

pub use block::BlockDevice;
pub use filesystem::FileSystemDriver;
pub use input::InputDriver;

use lucastra_core::Result;

/// Hardware Abstraction Layer trait for device operations.
pub trait HalDevice {
    fn probe(&mut self) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
}
