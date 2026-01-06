# LucAstra

**An augmented operating system built in Rust with embedded LLM and agentic capabilities.**

LucAstra is a prototype operating system that deeply integrates language models for natural language interaction, semantic search, and autonomous task execution via tools. Supports local inference (llamafile) and cloud APIs (OpenAI) with multi-turn conversations.

## Version 1.1.0 - Async LLM + Vector Search

**LucAstra v1.1.0 adds async LLM providers with vector-based semantic search!**

### New in v1.1.0
- Async LLM Provider Abstraction: Unified interface for OpenAI, llamafile, and future providers
- OpenAI Integration: GPT-4o-mini completions + text-embedding-3-small (1536-dim vectors)
- Vector Similarity Search: Cosine-based semantic search replacing keyword-only BM25
- Conversation Management: Multi-turn context with automatic windowing

### Previous Release (v1.0.0)
- Phase 1: HostFileAccess, audit logging, SecurityConfig (read/write/USB)
- Phase 2: relibc syscall handler, ELF loader, LibreOffice launcher
- Phase 3: Calculator + File Manager apps
- Phase 4: Lightweight browser (HTTP, HTML parsing, tabs, bookmarks)
- Phase 5: Observability (structured logging, metrics, integration tests)
- Phase 6: Release engineering (semver, CI/CD, Docker, packaging)

## Current Features

### Core OS
- Kernel boot and lifecycle management
- Hardware Abstraction Layer (HAL) with pluggable device drivers
- Device manager (USB, input devices)
- Filesystem manager with mount/unmount support
- Input event management
- Tracing and observability throughout

### AI & Search (v1.1.0)
- Async LLM Providers: OpenAI (GPT-4o-mini), llamafile (local 7B models)
- Vector Embeddings: OpenAI text-embedding-3-small (1536 dimensions)
- Semantic Search: Cosine similarity-based vector index
- BM25 Keyword Search: Full-text search with TF-IDF scoring
- Conversation Management: Multi-turn context with automatic windowing
- RAG Pipeline: Retrieval-Augmented Generation with context injection
- Graceful fallback to mock responses when LLM offline

### Compatibility Layer
- Relibc syscall handler (20+ syscalls)
- File descriptor table management
- FAT32 boot sector parser
- ELF header parser and validator

### GUI & Tools
- Desktop-style GUI with chat interface (iced)
- Taskbar with file manager button
- Color-coded chat messages
- Scrollable message history
- Search tool (BM25 filesystem search)
- Read tool (file contents)
- Install tool (execute commands, install programs)
- Tool execution framework for agentic tasks

## Quick Start (developers)

### Run the GUI
```powershell
cargo run --package lucastra-gui
```

What it does: boots kernel, initializes services, scans devices, mounts filesystem, indexes documents, and shows the chat interface.

### Run the CLI
```powershell
cargo run --package lucastra-app
```

### Try the tool demo
```powershell
cargo run --package lucastra-app --example tool_demo
```

### Configuration

#### LLM Provider Setup (v1.1.0)

**Option 1: OpenAI (Cloud)**
```bash
# Set API key
export OPENAI_API_KEY=sk-...

# Create config with OpenAI provider
cat > ~/.lucastra/config.json <<EOF
{
  "llm": {
    "provider": "openai",
    "api_key": "\${OPENAI_API_KEY}",
    "model": "gpt-4o-mini",
    "temperature": 0.7,
    "max_tokens": 4096
  },
  "embeddings": {
    "provider": "openai",
    "model": "text-embedding-3-small"
  }
}
EOF
```

**Option 2: Llamafile (Local)**
```bash
# Start llamafile server
llamafile -m mistral-7b-instruct.Q4_K_M.gguf --server --port 8000

# Use llamafile config
export LUCASTRA_CONFIG_HOME=./docs/examples/configs/dev.json
```

See [docs/examples/v1_1_async_llm.rs](docs/examples/v1_1_async_llm.rs) for complete examples.

#### Environment Variables
- LUCASTRA_CONFIG_HOME: Root directory for config.json (default: `~/.lucastra`)
- OPENAI_API_KEY: OpenAI API key (for OpenAI provider)
- RUST_LOG: Log level override (optional, respects config setting)

Logs live in `./logs` (file + console). Audit logs are JSON lines in `./audit/` when file operations occur.

## Project Structure

```
LucAstra/
├── kernel/         Boot coordination and lifecycle management
├── core/           Shared types (Command, Response, Error)
├── services/       Service registry framework
├── hal/            Hardware Abstraction Layer (device traits)
├── devices/        Device enumeration and management
├── fs/             Filesystem mounting and I/O routing
├── input/          Input event buffering
├── llm/            LLM integration (llamafile HTTP client)
├── search/         BM25 search service
├── compat/         Linux compatibility (relibc syscalls, ELF loader)
├── tools/          Agentic tools (search, read, install)
├── app/            CLI + library for system orchestration
├── gui/            Desktop GUI (iced)
└── db/             Database abstractions (future: LanceDB)
```

## Development

### Prerequisites
- Rust 1.90+ 
- PowerShell (Windows) or Bash (Linux/Mac)
- Optional: llamafile for LLM inference

### Building
```powershell
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build --package lucastra-gui

# Run tests
cargo test --workspace --lib
```

### Code Quality
```powershell
cargo fmt
cargo clippy --workspace
```

### LLM Setup (Optional)
1. Download llamafile (7B model)
2. Start server: `llamafile --server --port 8000`
3. LucAstra will connect automatically (falls back to mock if offline)

## Documentation

- [docs/OS_ARCHITECTURE.md](docs/OS_ARCHITECTURE.md) - System design and architecture
- [docs/MVP_SUMMARY.md](docs/MVP_SUMMARY.md) - MVP completion checklist
- [docs/GUI_TOOLS_GUIDE.md](docs/GUI_TOOLS_GUIDE.md) - GUI usage and tool API reference
**Release Documentation**:
- [docs/CHANGELOG.md](docs/CHANGELOG.md) - Complete feature history (v0.1.0 → v1.0.0)
- [docs/RELEASE_NOTES.md](docs/RELEASE_NOTES.md) - Deployment guide with configuration setup
- [docs/CONFIG_SCHEMA.md](docs/CONFIG_SCHEMA.md) - Configuration reference and best practices
- [docs/PHASE6_SUMMARY.md](docs/PHASE6_SUMMARY.md) - Release engineering completion summary

## Testing

All core functionality has been tested:
- Device enumeration (3 mock devices)
- Filesystem mounting and I/O
- BM25 search with document indexing
- RAG pipeline (search → context → LLM)
- Syscall handler (20+ syscalls)
- FAT32 and ELF parsing
- Tool execution (search, read, install)
- GUI chat interface
**v1.0.0 Test Results**:
- 68 Tests Total (all passing)
  - 53 unit tests (calculator, file manager, browser, security, observability)
  - 15 integration tests (system state, metrics, RAG pipeline, file access)
- 100% Pass Rate on Windows and Linux
- Security Audit: Clean dependency scan
- Code Quality: Clippy and rustfmt compliant

Run tests locally:
```bash
cargo test --lib          # Unit tests only
cargo test --test '*'     # Integration tests only
cargo test --all          # All tests
```

## Roadmap

### v0.2 (Next)
- [ ] Real device drivers (USB, keyboard, mouse)
- [ ] Persistent filesystem (beyond mock)
- [ ] Tool chaining and automation
- [ ] Permission system for tools
- [ ] Enhanced file manager GUI

### v0.3 (Future)
- [ ] LanceDB integration for vector search
- [ ] Run LibreOffice via relibc
- [ ] Custom tool API for user extensions
- [ ] Multi-window GUI support
- [ ] Real-time system monitoring

### v1.0 (Vision)
- [ ] Full Linux app compatibility
- [ ] Distributed LLM inference
- [ ] Plugin ecosystem
- [ ] Production-ready security
- [ ] Native hardware boot (no host OS)

**Current Status: v1.0.0 Production Release**

The roadmap above represents vision for v1.1, v1.2, and beyond. All core v1.0 features are complete and production-ready.

## Contributing

LucAstra is an experimental project. Contributions welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## Build & Deployment

### CI/CD Pipeline
GitHub Actions automatically:
- Lint code (rustfmt, clippy) on every push
- Build release binaries for Windows and Linux
- Run full test suite
- Scan dependencies for security vulnerabilities
- Generate code coverage reports
- Create releases on version tags

Pipeline configuration: `.github/workflows/ci.yml`

### Release Artifacts
Each release includes:
- Compiled binaries (Windows `.exe`, Linux binary)
- Source code
- Documentation and configuration samples

Download from GitHub Releases: https://github.com/[your-org]/LucAstra/releases

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Redox OS - relibc compatibility layer
- llamafile - Portable LLM inference
- iced - Rust GUI framework
- Rust community - Amazing ecosystem and tools