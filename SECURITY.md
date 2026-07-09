# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in WSForge, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email the maintainers or use GitHub's private vulnerability reporting
3. Include steps to reproduce and potential impact

## Security Considerations

WSForge is a network diagnostic tool. When using it:

- **Never** connect to untrusted WebSocket servers with sensitive data
- **Validate** all input file paths to prevent path traversal
- **Use** `--timeout` to prevent hanging connections
- **Review** captured session data before replaying against production systems

## Dependencies

WSForge uses well-maintained, audited dependencies:
- `tokio-tungstenite` — Async WebSocket client
- `tungstenite` — WebSocket protocol implementation
- `clap` — CLI argument parsing
- `serde` — Serialization/deserialization

All dependencies are pinned to specific versions for reproducibility.
