# LucAstra

LucAstra aims to prototype an augmented operating system built entirely in Rust, embedding an on-device LLM for natural language interaction, search, and system control. Everything runs locally: kernel/core services, GUI, local search (BM25-oriented, with room for vectors), and a local model.

## Vision
- OS acts as its own database; documents and state are searchable via BM25-style indexing with hooks for vector retrieval.
- Embedded LLM (7B class) runs locally for privacy-first interaction.
- Modular Rust workspace: kernel, services, database layer, and GUI (iced).
- Linux compatibility layer planned via Redox `relibc`.

## MVP scope (end-to-end, minimal)
- Boot kernel stub and start a core services layer.
- Local model directory for a 7B model (`model/`). No remote calls.
- Local database layer prepared for LanceDB; simple in-memory/BM25 placeholder first.
- GUI built with `iced` that can issue basic commands to services and display responses.
- No agentic/autonomous flows yet; just command in → response out.

## Architecture sketch
- `kernel/`: boot coordination, lifecycle hooks, minimal scheduling stubs.
- `services/`: registry for core services (command handling, search, IO shims).
- `db/`: abstraction over local search/backing store; plan to add LanceDB bindings.
- `gui/`: iced front-end to send commands and render responses.
- `model/`: place 7B weights/checkpoints here; keep out of git.

## Development workflow
- Use `cargo fmt` / `cargo clippy` before commits.
- Run individual crates: `cargo run -p lucastra-gui` for the GUI harness; libraries are wired as dependencies.
- Keep model files local-only (`model/` is gitignored).

## Next steps
1. Define data contracts between GUI, services, and DB (commands/responses).
2. Integrate LanceDB (or BM25 placeholder) behind a trait in `db/`.
3. Wire LLM runtime API to call into the model directory.
4. Add a compatibility layer experiment using `relibc` for Linux ABI surface.
5. Flesh out kernel lifecycle events and metrics via `tracing`.
