use tracing::info;

#[cfg(feature = "relibc")]
pub use lucastra_compat::SyscallHandler;

#[derive(Debug, Clone)]
pub struct KernelConfig {
    pub boot_message: &'static str,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            boot_message: "LucAstra kernel online",
        }
    }
}

pub fn boot(config: KernelConfig) {
    info!(message = config.boot_message, "Booting LucAstra kernel");

    #[cfg(feature = "relibc")]
    {
        if let Err(e) = lucastra_compat::init() {
            info!("Compatibility layer init skipped: {}", e);
        } else {
            info!("Compatibility layer initialized");
        }
    }
}

pub fn shutdown() {
    info!("Shutting down LucAstra kernel");
}
