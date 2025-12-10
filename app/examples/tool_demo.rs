use lucastra_app::SystemState;
use lucastra_tools::{Tool, InstallMethod};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
        )
        .init();

    println!("=== LucAstra Tool Execution Demo ===\n");

    // Initialize system
    let state = SystemState::new()?;

    // Test 1: Search tool
    println!("Test 1: Search Tool");
    let search_tool = Tool::Search {
        query: "LucAstra OS".to_string(),
        top_k: Some(3),
    };
    let result = state.execute_tool(search_tool);
    println!("Result: {:?}\n", result);

    // Test 2: Read tool
    println!("Test 2: Read Tool");
    let read_tool = Tool::Read {
        path: "/mnt/root/guide.txt".to_string(),
    };
    let result = state.execute_tool(read_tool);
    println!("Result: {:?}\n", result);

    // Test 3: Install tool (checking Rust installation)
    println!("Test 3: Install Tool - Check Rust version");
    let install_tool = Tool::Install {
        program: "rust".to_string(),
        method: InstallMethod::Command {
            cmd: "rustc".to_string(),
            args: vec!["--version".to_string()],
        },
    };
    let result = state.execute_tool(install_tool);
    println!("Result: {:?}\n", result);

    println!("=== Demo Complete ===");

    Ok(())
}
