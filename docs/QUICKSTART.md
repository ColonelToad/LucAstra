# LucAstra Quick Start Guide

Get up and running with LucAstra in 5 minutes!

## Prerequisites

- Windows 10/11 (Linux/Mac untested but may work)
- Rust 1.90 or newer
- PowerShell or Command Prompt
- 2GB RAM minimum (4GB+ recommended for LLM)

## Installation

### 1. Clone the Repository

```powershell
git clone https://github.com/yourusername/LucAstra.git
cd LucAstra
```

### 2. Build the Project

```powershell
cargo build --workspace --release
```

This will take 5-10 minutes on first build. Subsequent builds are much faster.

### 3. (Optional) Set Up LLM Server

If you want real LLM responses instead of mock responses:

1. Download llamafile from https://github.com/Mozilla-Ocho/llamafile
2. Start the server:
   ```powershell
   llamafile --server --port 8000
   ```
3. Wait for "Server listening on http://localhost:8000"

**Note**: LucAstra works fine without llamafile - it will fall back to mock responses.

## Running LucAstra

### Option 1: Desktop GUI (Recommended)

```powershell
cargo run --package lucastra-gui --release
```

You should see:
- A window titled "LucAstra OS - Desktop"
- A welcome message in the chat
- Input box at the bottom
- Taskbar with "File Manager" button

**Try these commands:**
- "What is LucAstra?"
- "Tell me about the architecture"
- "How does RAG work?"

### Option 2: Command Line Interface

```powershell
cargo run --package lucastra-app --release
```

Watch the boot sequence:
1. Kernel initialization
2. Device scanning
3. Filesystem mounting
4. Document indexing
5. Example commands execution

### Option 3: Tool Demo

```powershell
cargo run --package lucastra-app --example tool_demo --release
```

See all three tools in action:
- **Search**: Find documents with BM25
- **Read**: Access file contents
- **Install**: Execute commands (checks Rust version)

## Your First Session

### Using the GUI

1. **Start the GUI**
   ```powershell
   cargo run --package lucastra-gui --release
   ```

2. **Ask a question**
   - Type: "What is LucAstra?"
   - Press Enter or click Send
   - Wait for response (2-5 seconds with LLM, instant with mock)

3. **Try RAG-powered search**
   - Type: "Tell me about Rust integration"
   - The system will search indexed documents
   - Context is injected into LLM prompt
   - Get informed response with citations

4. **Explore the interface**
   - Click "File Manager" button (placeholder)
   - Scroll through message history
   - Notice color-coded messages (blue=you, green=assistant)

### Using the CLI

1. **Start the app**
   ```powershell
   cargo run --package lucastra-app --release
   ```

2. **Watch the output**
   - Boot sequence logs
   - Device enumeration (3 mock devices)
   - Filesystem mounting at /mnt/root
   - Document indexing (2 example docs)
   - Example commands (ListDevices, Search, Query, Echo)

3. **Check the results**
   - Device list displayed
   - Search results with scores
   - LLM response with RAG context
   - Echo message confirmation

## Troubleshooting

### Build Errors

**Problem**: "cannot find -lwgpu" or similar
```powershell
# Solution: Update Rust
rustup update
```

**Problem**: "linker error"
```powershell
# Solution: Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Select "Desktop development with C++"
```

### Runtime Errors

**Problem**: GUI window appears black
```
# Solution: Update graphics drivers
# Or try software rendering (slower):
set WGPU_BACKEND=dx12
cargo run --package lucastra-gui --release
```

**Problem**: "Connection refused" for LLM
```
# This is EXPECTED if llamafile isn't running
# LucAstra falls back to mock responses automatically
# No action needed unless you want real LLM responses
```

**Problem**: "Permission denied" for install tool
```
# Solution: Run PowerShell as Administrator
# Or adjust command permissions
```

### Performance Issues

**Problem**: GUI is slow/laggy
```
# Solution 1: Build in release mode
cargo run --package lucastra-gui --release

# Solution 2: Close other applications
# LLM inference is memory-intensive

# Solution 3: Disable LLM (use mock responses)
# Just don't start llamafile server
```

## Testing Everything Works

### Quick Smoke Test

```powershell
# Test 1: Build succeeds
cargo build --workspace
# Expected: "Finished dev profile" message

# Test 2: Tests pass
cargo test --workspace --lib
# Expected: "4 passed" message

# Test 3: GUI launches
cargo run --package lucastra-gui --release
# Expected: Window appears with chat interface

# Test 4: Tools work
cargo run --package lucastra-app --example tool_demo --release
# Expected: 3 tool results displayed
```

If all four tests pass, you're good to go! 🎉

## Next Steps

### Explore the Codebase

```
LucAstra/
├── gui/src/main.rs      Start here for GUI code
├── app/src/lib.rs       System state and command handling
├── tools/src/           Tool implementations
├── search/src/lib.rs    BM25 search algorithm
└── llm/src/lib.rs       LLM client integration
```

### Read the Documentation

- **GUI_TOOLS_GUIDE.md** - Detailed GUI and tool usage
- **OS_ARCHITECTURE.md** - System design and patterns
- **RELEASE_SUMMARY.md** - Feature overview
- **README.md** - Project vision and roadmap

### Try Advanced Features

1. **Custom Tool Execution**
   ```rust
   use lucastra_tools::{Tool, InstallMethod};
   
   let tool = Tool::Install {
       program: "my-tool".to_string(),
       method: InstallMethod::Command {
           cmd: "my-command".to_string(),
           args: vec!["--version".to_string()],
       },
   };
   
   let result = state.execute_tool(tool);
   ```

2. **RAG Pipeline Testing**
   - Index your own documents
   - Search with different queries
   - Observe context injection

3. **GUI Customization**
   - Modify colors in gui/src/main.rs
   - Add new buttons to taskbar
   - Customize chat message styles

### Contribute

Found a bug? Want to add a feature?

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Submit pull request

## Common Use Cases

### Use Case 1: Document Search

```rust
// Index documents
search_service.index_document(
    "/path/to/doc",
    "Your document content here..."
);

// Search
let results = search_service.search("query", 5)?;
```

### Use Case 2: Install Software

```rust
let tool = Tool::Install {
    program: "ripgrep".to_string(),
    method: InstallMethod::Command {
        cmd: "cargo".to_string(),
        args: vec!["install".to_string(), "ripgrep".to_string()],
    },
};
```

### Use Case 3: File Management

```rust
// List files
let files = filesystem.list_files("/mnt/root")?;

// Read file
let content = filesystem.read_file("/mnt/root/file.txt")?;

// Write file
filesystem.write_file("/mnt/root/new.txt", b"content")?;
```

## Getting Help

- **GitHub Issues**: Report bugs or request features
- **Documentation**: See `/docs` folder
- **Examples**: See `/app/examples`
- **Tests**: Check `*/tests/*.rs` for usage examples

## What's Next?

Check out the **roadmap** in README.md for upcoming features:
- Real device drivers
- Persistent filesystem
- Vector search with LanceDB
- LibreOffice integration
- Multi-window GUI
- And much more!

---

**Happy hacking with LucAstra! 🚀**
