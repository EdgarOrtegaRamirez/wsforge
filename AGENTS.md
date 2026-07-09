# AGENTS.md

## WSForge — WebSocket CLI & Protocol Analyzer

### Purpose
WSForge is a Rust CLI tool for analyzing, debugging, and validating WebSocket connections. It provides session capture, replay, schema validation, and type generation.

### Key Files
- `src/main.rs` — CLI entry point with subcommand dispatch
- `src/cli.rs` — Clap-based CLI argument definitions
- `src/client.rs` — WebSocket client using tokio-tungstenite
- `src/protocol.rs` — Core types: Session, Message, FrameType, Direction
- `src/replay.rs` — Session replay and report generation
- `src/validate.rs` — JSON schema validation engine
- `src/types.rs` — TypeScript/Python type generation from samples

### Architecture
- Async WebSocket client built on tokio + tokio-tungstenite
- Session captures include timestamps, frame types, and directions
- Schema validation supports object/array/string/number/boolean/enum types
- Type generation infers types from JSON samples

### Build & Test
```bash
cargo build --release
cargo test
cargo clippy -- -D warnings
```

### Dependencies
- tokio (async runtime)
- tokio-tungstenite (async WebSocket)
- tungstenite (WebSocket protocol)
- futures-util (stream/sink extensions)
- clap (CLI parsing)
- serde/serde_json (serialization)
- env_logger (logging)
