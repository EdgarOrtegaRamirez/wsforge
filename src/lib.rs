//! WSForge — Comprehensive WebSocket CLI and Protocol Analyzer
//!
//! A Rust CLI tool for analyzing, debugging, and validating WebSocket connections.
//! Features include:
//! - Connect and interact with WebSocket servers
//! - Capture and replay message sessions
//! - Validate messages against JSON schemas
//! - Auto-generate TypeScript/Python types from message samples
pub mod client;
pub mod protocol;
pub mod replay;
pub mod types;
pub mod validate;

pub use client::WsClient;
pub use protocol::{CloseFrame, FrameType, Message};
pub use replay::ReplaySession;
pub use types::{PythonType, TypeGenerator, TypeScriptType};
pub use validate::validate;
