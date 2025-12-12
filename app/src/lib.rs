use lucastra_config::Config;
use lucastra_core::{Command, CommandPayload, Response, ResponsePayload};
use lucastra_devices::DeviceManager;
use lucastra_fs::FilesystemManager;
use lucastra_hal::filesystem::MockFileSystem;
use lucastra_input::InputManager;
use lucastra_llm::LLMService;
use lucastra_search::SearchService;
use lucastra_services::ServiceRegistry;
use lucastra_tools::{
    file_access::{FileAccessTool, FileAccessValidator},
    install::InstallTool,
    read::ReadTool,
    search::SearchTool,
    Tool, ToolResult,
};
use std::path::Path;

pub mod metrics;
pub mod observability;
pub use metrics::{Metrics, MetricsSnapshot};

#[cfg(feature = "relibc")]
use lucastra_kernel::SyscallHandler;

/// System state holding all services.
pub struct SystemState {
    pub config: Config,
    pub service_registry: ServiceRegistry,
    pub device_manager: DeviceManager,
    pub filesystem: FilesystemManager,
    pub input_manager: InputManager,
    pub search_service: SearchService,
    pub llm_service: LLMService,
    pub metrics: Metrics,
    #[cfg(feature = "relibc")]
    pub syscall_handler: Option<SyscallHandler>,
}

impl SystemState {
    /// Initialize all services and boot the OS.
    pub fn new() -> lucastra_core::Result<Self> {
        tracing::info!("Initializing LucAstra system state");

        // Load configuration
        let config = Config::load().map_err(|e| {
            tracing::error!("Failed to load config: {}", e);
            lucastra_core::LuCastraError::ConfigError(format!("Config error: {}", e))
        })?;

        tracing::info!("Configuration loaded successfully");
        tracing::debug!("LLM server: {}", config.llm.server_url);
        tracing::debug!("Model size: {}", config.llm.model_size);
        tracing::debug!("Data directory: {}", config.storage.data_dir.display());

        let service_registry = ServiceRegistry::new();
        let mut device_manager = DeviceManager::new();
        let mut filesystem = FilesystemManager::new();
        let input_manager = InputManager::new();
        let mut search_service = SearchService::new();
        let llm_service = LLMService::new(config.llm.server_url.clone());

        // Scan devices
        device_manager.scan()?;
        tracing::info!("Found {} devices", device_manager.list_devices()?.len());

        // Mount mock filesystem
        let mock_fs = MockFileSystem::new();
        filesystem.mount("/mnt/root", mock_fs)?;

        // Index some example documents
        search_service.index_document(
            "/mnt/root/guide.txt",
            "LucAstra is an augmented OS with embedded LLM. It supports RAG for contextual responses.",
        )?;
        search_service.index_document(
            "/mnt/root/readme.txt",
            "LucAstra OS runs on Rust. It integrates with llamafile for 7B model inference.",
        )?;

        let metrics = Metrics::new();

        Ok(Self {
            config,
            service_registry,
            device_manager,
            filesystem,
            input_manager,
            search_service,
            llm_service,
            metrics,
            #[cfg(feature = "relibc")]
            syscall_handler: Some(SyscallHandler::new()),
        })
    }

    /// Get current configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Update configuration and save
    pub fn update_config(&mut self, new_config: Config) -> lucastra_core::Result<()> {
        new_config.save().map_err(|e| {
            lucastra_core::LuCastraError::ConfigError(format!("Failed to save config: {}", e))
        })?;

        self.config = new_config;
        tracing::info!("Configuration updated and saved");
        Ok(())
    }

    /// Handle a command and return a response.
    pub fn handle_command(&mut self, cmd: Command) -> lucastra_core::Result<Response> {
        match &cmd.payload {
            CommandPayload::ListDevices => {
                let devices = self.device_manager.list_devices()?;
                let device_strs = devices
                    .iter()
                    .map(|d| format!("{} ({})", d.path, d.name))
                    .collect::<Vec<_>>();
                Ok(Response {
                    command_id: cmd.id.clone(),
                    payload: ResponsePayload::Devices(device_strs),
                })
            }
            CommandPayload::Search { query } => {
                let results = self.search_service.search(query, 5)?;
                Ok(Response {
                    command_id: cmd.id.clone(),
                    payload: ResponsePayload::SearchResults(results),
                })
            }
            CommandPayload::Query { text, use_rag } => {
                let mut context = None;

                // Retrieve context if RAG is enabled
                if use_rag.unwrap_or(false) {
                    let search_results = self.search_service.search(text, 3)?;
                    context = Some(search_results.iter().map(|r| r.snippet.clone()).collect());
                }

                // Call LLM
                let response = self.llm_service.infer(lucastra_llm::InferenceRequest {
                    prompt: text.clone(),
                    max_tokens: Some(256),
                    temperature: Some(0.7),
                    context,
                })?;

                Ok(Response {
                    command_id: cmd.id.clone(),
                    payload: ResponsePayload::Success(response.text),
                })
            }
            CommandPayload::Status => Ok(Response {
                command_id: cmd.id.clone(),
                payload: ResponsePayload::Status(format!(
                    "LucAstra OS running. Devices: {}, Indexed docs: {}",
                    self.device_manager.list_devices()?.len(),
                    self.search_service.doc_count()
                )),
            }),
            CommandPayload::Echo { message } => Ok(Response {
                command_id: cmd.id.clone(),
                payload: ResponsePayload::Success(format!("Echo: {}", message)),
            }),
            _ => Ok(Response {
                command_id: cmd.id.clone(),
                payload: ResponsePayload::Success("Command not implemented".to_string()),
            }),
        }
    }

    /// Execute a tool (for agentic tasks).
    pub fn execute_tool(&self, tool: Tool) -> ToolResult {
        match tool {
            Tool::Search { query, top_k } => {
                let search_tool = SearchTool::new(&self.search_service);
                search_tool
                    .execute(&query, top_k.unwrap_or(5))
                    .unwrap_or_else(|e| ToolResult::failure("search", e.to_string()))
            }
            Tool::Read { path } => {
                let read_tool = ReadTool::new(&self.filesystem);
                read_tool
                    .execute(&path)
                    .unwrap_or_else(|e| ToolResult::failure("read", e.to_string()))
            }
            Tool::Install { program, method } => {
                let install_tool = InstallTool::new();
                install_tool
                    .execute(&program, &method)
                    .unwrap_or_else(|e| ToolResult::failure("install", e.to_string()))
            }
            Tool::HostFileAccess {
                operation,
                path,
                dest_path,
            } => {
                let validator = FileAccessValidator::new(
                    self.config.security.resolved_allowed_dirs(),
                    self.config.security.allow_host_read,
                    self.config.security.allow_host_write,
                    self.config.security.allow_usb,
                );

                let audit_path = match lucastra_config::get_logs_dir() {
                    Ok(dir) => dir.join("file_access_audit.log"),
                    Err(e) => {
                        return ToolResult::failure(
                            "host_file_access",
                            format!("Unable to resolve audit log dir: {}", e),
                        )
                    }
                };

                let tool = FileAccessTool::new(validator, audit_path);
                tool.execute(
                    operation,
                    Path::new(&path),
                    dest_path.as_deref().map(Path::new),
                )
            }
        }
    }

    /// Parse and execute tools from LLM JSON output.
    pub fn execute_tools_from_json(&self, json_str: &str) -> Vec<ToolResult> {
        let tools: Result<Vec<Tool>, _> = serde_json::from_str(json_str);

        match tools {
            Ok(tools) => tools
                .iter()
                .map(|tool| self.execute_tool(tool.clone()))
                .collect(),
            Err(e) => vec![ToolResult::failure(
                "parse",
                format!("Failed to parse tools: {}", e),
            )],
        }
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new().expect("Failed to initialize system state")
    }
}
