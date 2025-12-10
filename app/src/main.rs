use lucastra_app::SystemState;
use lucastra_core::{Command, CommandPayload};
use lucastra_kernel::KernelConfig;
use tracing::info;

fn main() -> lucastra_core::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
        )
        .init();

    info!("=== LucAstra OS Boot ===");

    // Boot kernel
    let config = KernelConfig::default();
    lucastra_kernel::boot(config);

    // Initialize system state
    let mut state = SystemState::new()?;

    // Initialize compat layer
    lucastra_compat::init()?;
    info!(
        "Compat layer enabled: relibc={}",
        lucastra_compat::is_relibc_enabled()
    );

    // Check LLM health
    info!("Checking LLM server health...");
    match state.llm_service.health_check() {
        Ok(true) => info!("LLM server is online"),
        Ok(false) => info!("LLM server is unreachable (expected if not running; will use mock responses)"),
        Err(e) => info!("LLM health check error (expected; will use mock): {}", e),
    }

    // Simulate a command loop (simplified for MVP)
    info!("=== Entering Command Loop ===");

    // Example 1: List devices
    let cmd = Command {
        id: "cmd-1".to_string(),
        payload: CommandPayload::ListDevices,
    };
    let response = state.handle_command(cmd)?;
    info!("Response: {:?}", response);

    // Example 2: Search
    let cmd = Command {
        id: "cmd-2".to_string(),
        payload: CommandPayload::Search {
            query: "LucAstra OS".to_string(),
        },
    };
    let response = state.handle_command(cmd)?;
    info!("Response: {:?}", response);

    // Example 3: Query with RAG
    let cmd = Command {
        id: "cmd-3".to_string(),
        payload: CommandPayload::Query {
            text: "What is LucAstra?".to_string(),
            use_rag: Some(true),
        },
    };
    let response = state.handle_command(cmd)?;
    info!("Response: {:?}", response);

    // Example 4: Echo
    let cmd = Command {
        id: "cmd-4".to_string(),
        payload: CommandPayload::Echo {
            message: "Hello from LucAstra!".to_string(),
        },
    };
    let response = state.handle_command(cmd)?;
    info!("Response: {:?}", response);

    info!("=== Boot Complete ===");
    info!("Ready for GUI or CLI interaction");

    Ok(())
}
