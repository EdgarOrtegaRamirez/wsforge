# WSForge — Comprehensive WebSocket CLI & Protocol Analyzer

A Rust CLI tool for analyzing, debugging, and validating WebSocket connections.

## Features

- **Connect & Interact** — Connect to WebSocket servers, send messages (text/binary), receive responses
- **Session Capture** — Capture full WebSocket sessions with timestamps, frame types, and directions
- **Session Replay** — Replay captured sessions with configurable speed and repetition
- **Schema Validation** — Validate messages against JSON schemas with detailed error reporting
- **Type Generation** — Auto-generate TypeScript/Python type definitions from message samples
- **Protocol Inspection** — Inspect WebSocket URLs and handshake details

## Installation

### From Source

```bash
cargo install --path .
```

### Build Locally

```bash
cargo build --release
cp target/release/wsforge ~/.local/bin/
```

## Quick Start

### Connect and Send a Message

```bash
wsforge connect --url wss://echo.websocket.org --message "Hello, WSForge!"
```

### Replay a Captured Session

```bash
wsforge replay --input session.json --url wss://echo.websocket.org --speed 2.0 --repeat 3
```

### Validate Messages Against a Schema

```bash
wsforge validate --schema schema.json --messages messages.jsonl --format markdown
```

### Generate Type Definitions

```bash
wsforge types --input samples.jsonl --language typescript --name ChatMessage
wsforge types --input samples.jsonl --language python --name ChatMessage
```

### Inspect a WebSocket URL

```bash
wsforge inspect --url wss://api.example.com/ws
```

### Format a Captured Session

```bash
wsforge format --input session.json --format markdown
```

## Schema Format

WSForge uses a simplified JSON schema format:

```json
{
  "type": "object",
  "required": ["type", "data"],
  "properties": {
    "type": {"type": "string"},
    "data": {"type": "string"},
    "timestamp": {"type": "number"}
  }
}
```

Supported types: `string`, `number`, `boolean`, `object`, `array`, `enum`

## Session Format

Sessions are stored as JSON with this structure:

```json
{
  "url": "wss://echo.websocket.org",
  "messages": [
    {
      "frame_type": "Text",
      "payload": "Hello!",
      "timestamp": "2024-01-01T00:00:00Z",
      "direction": "Sent"
    }
  ],
  "started_at": "2024-01-01T00:00:00Z",
  "ended_at": null,
  "close_frame": null
}
```

## CLI Reference

```
wsforge <COMMAND>

Commands:
  connect   Connect to a WebSocket server
  replay    Replay a captured session
  validate  Validate messages against a JSON schema
  types     Generate type definitions from samples
  inspect   Inspect a WebSocket URL
  format    Format and beautify a captured session
  help      Print this message
```

## Architecture

- **client.rs** — WebSocket client with async connect/send/receive
- **protocol.rs** — Core types: Session, Message, FrameType, Direction
- **replay.rs** — Session replay and report generation
- **validate.rs** — JSON schema validation engine
- **types.rs** — Type definition generation (TypeScript/Python)
- **cli.rs** — CLI argument parsing with clap

## License

MIT
