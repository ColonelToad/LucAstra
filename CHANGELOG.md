# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-12-10

### Added

#### Phase 1: File I/O & Sandboxing
- **HostFileAccess Tool** with sandboxed file operations
  - FileAccessValidator with whitelist-based access control
  - Audit logging to JSON-line format with timestamp tracking
  - Operations: Read, Write, Copy, Move, Delete, List
  - Support for allow_host_read, allow_host_write, allow_usb flags
- **SecurityConfig** in config module
  - Configurable allowed_host_dirs with tilde (~) expansion
  - User-approvable operations
  - Automatic audit entry generation
- 7 unit tests for file access and audit logging

#### Phase 2: relibc Integration & Compatibility Layer
- **relibc Feature Flag** in kernel with conditional compilation
  - ELF loader with program header parsing
  - FAT32 boot sector support
  - Syscall handler with file descriptor table
- **LibreOffice Launcher** with sandbox toggle
  - Document path validation
  - Executable path resolution
  - Sandboxed process execution support
- Conditional compilation for compatibility layer
- 8 unit tests for syscall handling and ELF loading

#### Phase 3: Native Applications
- **Calculator App** (apps/calculator)
  - Expression parser with operator precedence
  - Support for: +, -, *, /, (, ), sqrt, sin, cos, tan, abs, ln, log
  - History tracking
  - Division by zero error handling
  - 9 unit tests
- **File Manager App** (apps/file-manager)
  - Directory navigation with history
  - File listing with metadata (size, modified time)
  - Copy, move, delete operations
  - Atomic file operations
  - 5 unit tests

#### Phase 4: Lightweight Browser
- **HttpClient** with blocking reqwest
  - User-Agent: LucAstra-Browser/1.0
  - 10-second timeout
  - URL validation
- **HtmlParser** with regex-based extraction
  - Title extraction from `<title>` tags
  - Text content extraction (strip HTML, skip script/style)
  - Link extraction with href and text
  - Image extraction with src attributes
  - HTML entity decoding (&lt;, &gt;, &quot;, etc.)
- **Browser UI** with tab management
  - Multiple tabs with independent history
  - Back navigation per tab
  - Bookmarks storage
  - URL-based routing
  - Interactive CLI with commands: open, tab, close, back, bookmark, bookmarks, tabs, exit
- 7 unit tests covering HTTP, parsing, and tab management

#### Phase 5: Quality & Observability
- **Tracing Module** (app/src/observability.rs)
  - `init_tracing()` with rolling daily file appender
  - Configurable log levels: trace, debug, info, warn, error
  - Dual output: file (lucastra.log) + console
  - Pretty formatting with span events and targets
  - Integration-ready for boot sequence
- **Metrics Collection** (app/src/metrics.rs)
  - Thread-safe Metrics struct using Arc<AtomicU64>
  - Tracked metrics: commands, tool success/failure, search queries/latency
  - MetricsSnapshot for reporting (Serde serializable)
  - Methods: record_command(), record_tool_success(), record_search()
  - Utility: tool_success_rate() percentage calculation
- **Config Enhancements**
  - TracingConfig: level, file_logging, log_dir, max_log_size_mb, log_files_keep, console_output, json_format
  - MetricsConfig: enabled, export_to_file, export_dir, export_interval_secs
  - Sensible defaults (info level, 10 MB max size, 5 file retention)
- **Integration Tests** (15 total)
  - SystemState initialization and config persistence
  - Metrics tracking across tool execution
  - HostFileAccess integration with validator/tool/audit
  - RAG pipeline readiness (search + LLM)
  - Security config whitelist verification
  - Filesystem operations and mounts
- 53 unit tests + 15 integration tests = 68 total passing

#### Phase 6: Release Engineering
- **Semantic Versioning**: 1.0.0
  - Workspace version bumped to 1.0.0
  - Config version updated to match
  - Ready for production distribution
- **Sample Configurations**
  - Development config with debug logging and file metrics export
  - Production config with info logging, disabled metrics export
  - Minimal config for resource-constrained environments
  - Each includes documented security settings
- **Build Artifacts**
  - Release build configuration
  - Feature matrix for optional components (relibc)
  - Dependency audit ready
- **Dependency Audit**
  - Clean build with no security warnings
  - All dependencies verified and documented
- **Documentation**
  - CHANGELOG documenting all phases and features
  - README with setup instructions
  - Configuration schema documentation
  - Release notes for v1.0.0

### Changed
- Version bumped from 0.1.0 to 1.0.0
- Config version bumped from 0.2.0 to 1.0.0
- File manager tests use unique temp directories to prevent race conditions

### Fixed
- File manager test isolation issues with concurrent runs
- Unused imports in browser and app modules
- Tracing module naming conflict resolution

### Security
- HostFileAccess tool enforces whitelist-based file access
- Audit logging for all file operations
- SecurityConfig provides fine-grained control
- User approval workflow for file operations
- Sandboxed relibc with syscall interception

### Performance
- Metrics collection uses lock-free atomic operations
- Async-ready tracing infrastructure
- Efficient regex-based HTML parsing
- Blocking HTTP client suitable for single-threaded apps

## [0.1.0] - Initial Development

Initial development phase with foundational kernel, core services, and infrastructure.
