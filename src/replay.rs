//! Session replay functionality
//!
//! Replay captured WebSocket sessions with configurable timing.

use crate::protocol::Session;
use serde_json;
use log::{info, debug};

/// Replay a captured session against a WebSocket server
pub struct ReplaySession {
    pub session: Session,
    pub speed: f64,
    pub repeat: u32,
}

impl ReplaySession {
    pub fn new(session: Session, speed: f64, repeat: u32) -> Self {
        Self { session, speed, repeat }
    }

    /// Print a summary of the session
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Session: {}", self.session.url));
        lines.push(format!("Messages: {}", self.session.message_count()));
        lines.push(format!("Started: {}", self.session.started_at));
        lines.push(format!("Ended: {:?}", self.session.ended_at));

        if let Some(cf) = &self.session.close_frame {
            lines.push(format!("Close: {} ({})", cf.code, cf.reason));
        }

        let sent = self.session.messages.iter().filter(|m| m.direction == crate::protocol::Direction::Sent).count();
        let received = self.session.messages.iter().filter(|m| m.direction == crate::protocol::Direction::Received).count();
        lines.push(format!("Sent: {} | Received: {}", sent, received));

        lines.join("\n")
    }

    /// Format session as markdown report
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# WebSocket Session Report\n\n");
        md.push_str(&format!("**URL:** {}\n\n", self.session.url));
        md.push_str(&format!("**Messages:** {}\n\n", self.session.message_count()));
        md.push_str("| # | Direction | Type | Payload |\n");
        md.push_str("|---|-----------|------|---------|\n");

        for (i, msg) in self.session.messages.iter().enumerate() {
            let payload = if msg.payload.len() > 100 {
                format!("{}...", &msg.payload[..100])
            } else {
                msg.payload.clone()
            };
            let escaped = payload.replace("|", "\\|").replace('\n', " ");
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                i + 1,
                msg.direction,
                msg.frame_type,
                escaped
            ));
        }

        md
    }

    /// Export session as JSON
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.session)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    /// Parse a session from JSON
    pub fn from_json(json: &str) -> Result<Session, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse session JSON: {}", e))
    }

    /// Validate that a session has at least one message
    pub fn is_empty(&self) -> bool {
        self.session.message_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Message as WsMessage, FrameType, Direction};

    #[test]
    fn test_session_summary() {
        let mut session = Session::new("ws://example.com");
        session.add_message(FrameType::Text, "hello".to_string(), Direction::Sent);
        session.add_message(FrameType::Text, "world".to_string(), Direction::Received);

        let replay = ReplaySession::new(session, 1.0, 1);
        let summary = replay.summary();
        assert!(summary.contains("ws://example.com"));
        assert!(summary.contains("2"));
        assert!(summary.contains("Sent: 1"));
        assert!(summary.contains("Received: 1"));
    }

    #[test]
    fn test_session_to_json() {
        let mut session = Session::new("ws://example.com");
        session.add_message(FrameType::Text, "test".to_string(), Direction::Sent);

        let replay = ReplaySession::new(session, 1.0, 1);
        let json = replay.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["messages"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_session_from_json() {
        let json = r#"{
            "url": "ws://example.com",
            "messages": [
                {
                    "frame_type": "Text",
                    "payload": "hello",
                    "timestamp": "2024-01-01T00:00:00Z",
                    "direction": "Sent"
                }
            ],
            "started_at": "2024-01-01T00:00:00Z",
            "ended_at": null,
            "close_frame": null
        }"#;

        let session = ReplaySession::from_json(json).unwrap();
        assert_eq!(session.url, "ws://example.com");
        assert_eq!(session.message_count(), 1);
    }

    #[test]
    fn test_markdown_report() {
        let mut session = Session::new("ws://example.com");
        session.add_message(FrameType::Text, "test message".to_string(), Direction::Sent);

        let replay = ReplaySession::new(session, 1.0, 1);
        let md = replay.to_markdown();
        assert!(md.contains("WebSocket Session Report"));
        assert!(md.contains("test message"));
    }

    #[test]
    fn test_empty_session() {
        let session = Session::new("ws://example.com");
        let replay = ReplaySession::new(session, 1.0, 1);
        assert!(replay.is_empty());
    }
}
