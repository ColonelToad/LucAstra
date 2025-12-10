# LucAstra

**An augmented operating system built in Rust with embedded LLM and agentic capabilities.**

LucAstra is a prototype operating system that deeply integrates a local 7B parameter language model for natural language interaction, intelligent search (BM25), and autonomous task execution via tools. Everything runs locally for privacy and control.

## 🎯 Vision
- **OS-as-Database**: Documents and system state searchable via BM25 indexing with future vector support
- **Embedded LLM**: 7B class model (llamafile) runs locally for privacy-first AI interaction
- **Agentic Tools**: Search, read, install programs - all controllable by natural language
- **Modular Architecture**: Clean Rust workspace with kernel, services, GUI, HAL, and tools
- **Linux Compatibility**: Run Linux binaries via Redox `relibc` compatibility layer (experimental)

## ✨ Current Features (MVP Complete!)

### Core OS
- ✅ Kernel boot and lifecycle management
- ✅ Hardware Abstraction Layer (HAL) with pluggable device drivers
- ✅ Device manager (USB, input devices)
- ✅ Filesystem manager with mount/unmount support
- ✅ Input event management
- ✅ Tracing and observability throughout

### AI & Search
- ✅ BM25-based document search service
- ✅ LLM integration via llamafile HTTP API
- ✅ RAG (Retrieval-Augmented Generation) pipeline
- ✅ Graceful fallback to mock responses when LLM offline

### Compatibility Layer
- ✅ Relibc syscall handler (20+ syscalls)
- ✅ File descriptor table management
- ✅ FAT32 boot sector parser
- ✅ ELF header parser and validator

### GUI & Tools
- ✅ Desktop-style GUI with chat interface (iced)
- ✅ Taskbar with file manager button
- ✅ Color-coded chat messages
- ✅ Scrollable message history
- ✅ Search tool (BM25 filesystem search)
- ✅ Read tool (file contents)
- ✅ Install tool (execute commands, install programs)
- ✅ Tool execution framework for agentic tasks

## 🚀 Quick Start

### Running the GUI
```powershell
cargo run --package lucastra-gui
```

The GUI will:
1. Boot the kernel
2. Initialize all services
3. Scan for devices
4. Mount filesystem
5. Index example documents
6. Display chat interface

### Running the CLI
```powershell
cargo run --package lucastra-app
```

### Testing Tools
```powershell
cargo run --package lucastra-app --example tool_demo
```

## 📦 Project Structure

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

## 🛠️ Development

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

## 📚 Documentation

- **[OS_ARCHITECTURE.md](OS_ARCHITECTURE.md)** - System design and architecture
- **[MVP_SUMMARY.md](MVP_SUMMARY.md)** - MVP completion checklist
- **[GUI_TOOLS_GUIDE.md](GUI_TOOLS_GUIDE.md)** - GUI usage and tool API reference

## 🧪 Testing

All core functionality has been tested:
- ✅ Device enumeration (3 mock devices)
- ✅ Filesystem mounting and I/O
- ✅ BM25 search with document indexing
- ✅ RAG pipeline (search → context → LLM)
- ✅ Syscall handler (20+ syscalls)
- ✅ FAT32 and ELF parsing
- ✅ Tool execution (search, read, install)
- ✅ GUI chat interface

## 🎯 Roadmap

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

## 🤝 Contributing

LucAstra is an experimental project. Contributions welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## 📄 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- **Redox OS** - relibc compatibility layer
- **llamafile** - Portable LLM inference
- **iced** - Rust GUI framework
- **Rust community** - Amazing ecosystem and tools
