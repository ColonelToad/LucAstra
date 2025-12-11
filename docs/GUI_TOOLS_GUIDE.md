# LucAstra GUI & Tools Guide

## Overview

LucAstra now includes a complete desktop-style GUI with chat interface and agentic tool support. This guide covers the new features added in this release.

## GUI Features

### Desktop Interface

The LucAstra GUI provides a desktop-style experience with:

- **Chat Interface**: Interactive chat with the embedded LLM in the center of the screen
- **Taskbar**: Bottom taskbar with quick access to system features
- **File Manager Button**: Easy access to file management (placeholder in MVP)
- **Scrollable Message History**: View all your interactions with the system
- **Color-Coded Messages**: 
  - User messages: Blue
  - LucAstra responses: Green
  - System messages: Gray

### Running the GUI

```powershell
# From the LucAstra directory
cargo run --package lucastra-gui
```

The GUI automatically:
1. Boots the kernel
2. Initializes all services (device manager, filesystem, search, LLM)
3. Scans for devices
4. Mounts the filesystem
5. Indexes example documents
6. Displays welcome message

### Using the Chat Interface

1. Type your message in the input box at the bottom
2. Press Enter or click "Send"
3. LucAstra will process your query using RAG (Retrieval-Augmented Generation)
4. The response appears in the chat history

Example queries:
- "What is LucAstra?"
- "Tell me about the OS architecture"
- "How does the LLM integration work?"

## Agentic Tools

LucAstra supports tool-based execution for autonomous tasks. Tools can be executed directly or parsed from LLM JSON output.

### Available Tools

#### 1. Search Tool
Searches the filesystem using BM25 algorithm.

```rust
use lucastra_tools::Tool;

let search_tool = Tool::Search {
    query: "LucAstra OS".to_string(),
    top_k: Some(5),  // Return top 5 results
};

let result = state.execute_tool(search_tool);
```

#### 2. Read Tool
Reads file contents from the filesystem.

```rust
let read_tool = Tool::Read {
    path: "/mnt/root/guide.txt".to_string(),
};

let result = state.execute_tool(read_tool);
```

#### 3. Install Tool
Executes commands to install programs.

**Method 1: Direct Command**
```rust
let install_tool = Tool::Install {
    program: "rust".to_string(),
    method: InstallMethod::Command {
        cmd: "rustc".to_string(),
        args: vec!["--version".to_string()],
    },
};
```

**Method 2: Download and Install**
```rust
let install_tool = Tool::Install {
    program: "myapp".to_string(),
    method: InstallMethod::Download {
        url: "https://example.com/installer.exe".to_string(),
        installer_args: vec!["/silent".to_string()],
    },
};
```

### Tool Execution API

The `SystemState` struct provides two methods for tool execution:

```rust
// Execute a single tool
pub fn execute_tool(&self, tool: Tool) -> ToolResult;

// Parse and execute tools from LLM JSON output
pub fn execute_tools_from_json(&self, json_str: &str) -> Vec<ToolResult>;
```

### Tool Result Format

All tools return a `ToolResult`:

```rust
pub struct ToolResult {
    pub tool: String,      // Tool name
    pub success: bool,     // Whether execution succeeded
    pub output: String,    // Result or error message
}
```

### Testing Tools

Run the tool demo to see all tools in action:

```powershell
cargo run --package lucastra-app --example tool_demo
```

This demo:
1. Searches for "LucAstra OS" documents
2. Reads a file from the filesystem
3. Checks Rust installation (via install tool)

## Program Installation Examples

### Example 1: Check Rust Installation

```rust
let tool = Tool::Install {
    program: "rust".to_string(),
    method: InstallMethod::Command {
        cmd: "rustc".to_string(),
        args: vec!["--version".to_string()],
    },
};
```

### Example 2: Install a Package with Cargo

```rust
let tool = Tool::Install {
    program: "ripgrep".to_string(),
    method: InstallMethod::Command {
        cmd: "cargo".to_string(),
        args: vec!["install".to_string(), "ripgrep".to_string()],
    },
};
```

### Example 3: Run PowerShell Script

```rust
let tool = Tool::Install {
    program: "custom-setup".to_string(),
    method: InstallMethod::Command {
        cmd: "powershell".to_string(),
        args: vec![
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            "setup.ps1".to_string(),
        ],
    },
};
```

## Integration with LLM

The LLM can be instructed to use tools by returning JSON in a specific format:

```json
[
  {
    "tool": "Search",
    "params": {
      "query": "LucAstra architecture",
      "top_k": 3
    }
  },
  {
    "tool": "Read",
    "params": {
      "path": "/mnt/root/guide.txt"
    }
  }
]
```

The system will parse this JSON and execute each tool in sequence.

## Architecture

### GUI → SystemState → Services Flow

```
┌─────────────┐
│   GUI       │
│  (iced)     │
└──────┬──────┘
       │
       │ Message::SendMessage
       ▼
┌─────────────┐
│ SystemState │
│             │
├─────────────┤
│ • handle_command()
│ • execute_tool()
└──────┬──────┘
       │
       ├──► DeviceManager
       ├──► FilesystemManager
       ├──► SearchService
       ├──► LLMService
       └──► Tools (Search, Read, Install)
```

### Tool Execution Flow

```
Tool Request
     │
     ▼
execute_tool()
     │
     ├─── SearchTool
     │       └─► BM25 Index
     │
     ├─── ReadTool
     │       └─► FilesystemManager
     │
     └─── InstallTool
             └─► Process::Command
                    └─► PowerShell/Executable
```

## Future Extensions

### Planned Features

1. **File Manager**: Full GUI file browser
2. **Installation Wizard**: Visual interface for program installation
3. **Tool Chaining**: Execute multiple tools in sequence automatically
4. **LibreOffice Integration**: Run Linux apps via relibc
5. **Custom Tool API**: Allow users to define their own tools
6. **Tool History**: Track and replay tool executions
7. **Permission System**: Control which tools can be executed

### Adding New Tools

To add a new tool:

1. Define the tool variant in `tools/src/lib.rs`:
```rust
pub enum Tool {
    // Existing tools...
    
    /// Your new tool
    MyTool { param1: String, param2: i32 },
}
```

2. Create implementation in `tools/src/mytool.rs`:
```rust
pub struct MyTool;

impl MyTool {
    pub fn execute(&self, param1: &str, param2: i32) -> Result<ToolResult> {
        // Implementation
    }
}
```

3. Add execution case in `app/src/lib.rs`:
```rust
Tool::MyTool { param1, param2 } => {
    let tool = MyTool::new();
    tool.execute(&param1, param2)
        .unwrap_or_else(|e| ToolResult::failure("mytool", e.to_string()))
}
```

## Testing LibreOffice (Future)

Once the ELF loader is fully integrated:

```rust
// Download LibreOffice for Linux
let download_tool = Tool::Install {
    program: "libreoffice".to_string(),
    method: InstallMethod::Download {
        url: "https://download.documentfoundation.org/libreoffice/...",
        installer_args: vec![],
    },
};

// Execute via relibc compatibility layer
// This will use the ELF loader and syscall handler
```

## Troubleshooting

### GUI won't start
- Ensure wgpu is properly installed
- Check that your graphics drivers support Vulkan or DX12
- Try running with `RUST_LOG=debug` for more info

### LLM not responding
- Verify llamafile is running at `http://localhost:8000`
- Check network connectivity
- System will fall back to mock responses if LLM is unavailable

### Install tool fails
- Check that the command exists in PATH
- Verify permissions for executing the command
- Review error output in the ToolResult

### Search returns no results
- Ensure documents are indexed
- Check that the query terms exist in documents
- Try broader search terms

## Performance Notes

- GUI rendering: ~60 FPS on modern hardware
- Search latency: <10ms for indexed corpus
- LLM inference: 2-5 seconds (depends on llamafile)
- Tool execution: Varies by tool (commands ~100ms, downloads slower)

## Security Considerations

**⚠️ WARNING**: The install tool executes arbitrary commands with your user privileges. In production:

1. Add permission checks before execution
2. Implement command whitelisting
3. Sandbox tool execution
4. Log all tool invocations
5. Require user confirmation for sensitive operations

Current MVP has NO security restrictions - use only for testing!
