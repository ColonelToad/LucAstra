use std::collections::HashSet;

use thiserror::Error;
use tracing::info;

pub type ServiceResult<T> = Result<T, ServiceError>;

pub trait Service {
    fn name(&self) -> &str;
    fn start(&mut self) -> ServiceResult<()>;
}

pub struct ServiceRegistry {
    started: HashSet<String>,
    services: Vec<Box<dyn Service + Send>>, // simple registry for now
}

#[allow(clippy::derivable_impls)]
impl Default for ServiceRegistry {
    fn default() -> Self {
        Self {
            started: HashSet::new(),
            services: Vec::new(),
        }
    }
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, service: Box<dyn Service + Send>) -> ServiceResult<()> {
        if self.started.contains(service.name()) {
            return Err(ServiceError::AlreadyStarted(service.name().into()));
        }
        self.services.push(service);
        Ok(())
    }

    pub fn start_all(&mut self) -> ServiceResult<()> {
        for service in self.services.iter_mut() {
            info!(service = service.name(), "Starting service");
            service.start()?;
            self.started.insert(service.name().into());
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("service already started: {0}")]
    AlreadyStarted(String),
    #[error("service failed: {0}")]
    Failed(String),
}
