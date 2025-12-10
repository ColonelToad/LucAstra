use tracing::info;

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
}

pub fn shutdown() {
    info!("Shutting down LucAstra kernel");
}
