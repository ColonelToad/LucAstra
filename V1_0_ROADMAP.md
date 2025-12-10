# LucAstra v1.0 Roadmap

**Target**: Production-ready public release
**Scope**: Stable desktop experience with real hardware, persistence, security, and observability

---

## Core Pillars

### 1. Hardware, Persistence & File I/O
- [ ] Replace mock filesystem with persistent backend (local dirs under `~/.lucastra/data/`)
- [ ] Wire HAL to real USB device enumeration (keyboard, mouse, mass storage)
- [ ] GPU detection + capability flag for acceleration decisions
- [ ] HostFileAccess tool: read/write/move from host dirs with whitelist + audit log
- [ ] USB sandboxing: detect + prompt user, store allowed paths in config
- [ ] Auto-sync setting: config toggle (default: off; explicit copy only)
- [ ] Create installer/portable ZIP with sample config and data dirs

### 2. Security & Permissions
- [ ] RBAC: user roles (admin, power-user, guest) with tool allowlists
- [ ] Tool sandboxing: cwd jail + capability whitelist per tool
- [ ] Audit log: log all tool executions and config changes to file
- [ ] Optional biometric auth: Windows Hello / Touch ID stubs with config toggle
- [ ] Config validation: reject invalid inputs at parse time

### 3. Native & Bundled Applications
- [ ] **relibc integration**: Build relibc, wire syscall handler → LibreOffice launcher
- [ ] **LibreOffice**: Optional download (not bundled); prove relibc sandbox + I/O bridge
- [ ] **Lightweight browser**: HTML renderer (webkit stub or HTTP client + simple DOM)
- [ ] **Calculator**: Native Rust/iced GUI (basic arithmetic + functions)
- [ ] **File Manager**: List/open/copy/move/delete with confirmation; drag-drop support
- [ ] **Text Editor**: Syntax highlighting (syntect), find/replace, line numbers, auto-save
- [ ] **PDF Viewer**: Zoom/scroll/search via pdfium-render or pdf crate
- [ ] **Terminal Emulator**: Shell out to `cmd.exe` / `/bin/bash` via PTY; capture output in GUI

### 4. RAG & LLM
- [ ] Configurable corpus roots (data/, models/) with lazy ingestion
- [ ] Deterministic chunking with metadata (source, mtime, size)
- [ ] Dual-path retrieval: BM25 today + pluggable vector backend (behind feature flag)
- [ ] LLM health checks + exponential backoff on failure
- [ ] Response cache + timeout controls (TTL, max_age config)
- [ ] Offline mode: graceful fallback when LLM unavailable

### 4. User Experience
- [ ] Polish toast/banner styling (auto-dismiss after 5s option)
- [ ] Resilient settings load/save with validation feedback
- [ ] App launcher: grid/list view with categories (built-in, optional, dev tools)
- [ ] Status tray: connectivity indicator, LLM state, log path shortcut, USB devices
- [ ] Keyboard shortcuts (Ctrl+,, Ctrl+Q, Alt+Tab between apps)
- [ ] Dark/light theme toggle with persist
- [ ] Window management: minimize/maximize/snap (OS integration)

### 5. Quality & Observability
- [ ] Tracing levels via config (error/warn/info/debug/trace)
- [ ] Rolling file appender with size rotation
- [ ] Metrics hooks: command count, search latency, tool success rate, app startup time
- [ ] Integration tests: config paths, persistence, tool sandbox, HostFileAccess, RAG pipeline
- [ ] GUI snapshot/golden tests (iced UI testing)
- [ ] relibc + syscall handler tests (file ops, process spawning)
- [ ] CI matrix: Windows + WSL2 Linux, all tests + clippy + audit

### 6. Release Engineering
- [ ] Semantic versioning + CHANGELOG.md
- [ ] Release notes per version
- [ ] Signed artifacts: ZIP (portable), MSI (installer)
- [ ] Sample configs showing `LUCASTRA_CONFIG_HOME` override
- [ ] Upgrade path: old config migration + versioned schema
- [ ] Dependency audit + license check

---

## Success Criteria

- ✅ Boots and runs without crashes after 10min continuous use
- ✅ Settings persist across restarts
- ✅ File operations work (read, write, delete, list)
- ✅ LLM responds within 5s or gracefully fails
- ✅ Tool execution logged and sandboxed
- ✅ No security warnings from clippy/audit
- ✅ Windows + Linux CI passing
- ✅ README + docs explain LUCASTRA_CONFIG_HOME and config schema
- ✅ At least 3 sample configs (dev, prod, minimal)

---

## Timeline & Milestones

### Phase 1: File I/O & Sandboxing (1-2 sprints)
- Tasks: HostFileAccess tool, whitelist system, USB detection, audit logging, config schema
- Owner: yourself
- Estimated: 1-2 weeks

### Phase 2: relibc Integration (2-3 sprints)
- Tasks: relibc build, syscall handler wiring, LibreOffice launcher + packaging, I/O bridge tests
- Owner: yourself + relibc review
- Estimated: 2-3 weeks

### Phase 3: Native Apps (2-3 sprints)
- Tasks: Calculator, File Manager, Text Editor (with syntax highlight), PDF Viewer, Terminal Emulator
- Owner: yourself
- Estimated: 2-3 weeks

### Phase 4: Lightweight Browser (1-2 sprints)
- Tasks: HTTP client + HTML renderer, tabs, history, bookmarks, network sandbox
- Owner: yourself
- Estimated: 1-2 weeks

### Phase 5: Quality (Tests, CI, Metrics) (1-2 sprints)
- Tasks: integration tests, GitHub Actions (Windows + WSL2), observability hooks, docs
- Owner: yourself
- Estimated: 1-2 weeks

### Phase 6: Release (Signing, Packaging, Docs) (1 sprint)
- Tasks: versioning, changelog, installer + ZIP artifacts, sample configs, upload
- Owner: yourself
- Estimated: 1 week

**Total: ~10-12 weeks (~2.5-3 months) with focused effort.**

---

## Decision Points

1. **File Storage**: Local dirs only (no SQLite) for simplicity and speed. ✅
2. **Vector Search**: Manual BM25 for v1.0, feature-flagged LanceDB for v1.1. ✅
3. **Auth Method**: Windows Hello / Touch ID stubs (config-gated). ✅
4. **Installer Format**: Both MSI (easy install) + portable ZIP. ✅
5. **Linux Support**: WSL2 during dev, CI gate on Windows + WSL2. ✅
6. **relibc Scope**: Full integration; LibreOffice as optional download (not bundled). ✅
7. **Browser**: Lightweight (webkit stub or HTTP client + simple HTML renderer). ✅
8. **IDE**: Skip for v1.0; defer to v1.1. ✅
9. **Terminal**: Shell out to OS (`cmd.exe` / `/bin/bash`); capture output in GUI. ✅
10. **Auto-sync**: Config setting; default off (explicit copy/move only). ✅

---

## Non-Goals (v1.0)

- IDE or advanced code editor (defer to v1.1)
- Git GUI or version control integration
- Distributed inference or model serving
- Plugin API (defer to v1.1)
- Real-time collaboration features
- Android/iOS ports
- Custom LLM fine-tuning
- Multi-user system accounts
- Cloud sync (iCloud, OneDrive, S3)

---

## Backlog (Post-v1.0)

- LanceDB vector store + semantic search
- IDE or lightweight code editor (helix, zed)
- Git GUI + version control integration
- Plugin system with capability model
- Advanced tool composition (chain → map → reduce)
- Performance profiler GUI
- Telemetry aggregation service
- Mobile companion apps
- Real-time collaboration (CRDT-based)

