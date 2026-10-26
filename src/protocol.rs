//! Protocol-level types for WebSocket frames

use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of WebSocket frames
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameType {
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

impl fmt::Display for FrameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameType::Text => write!(f, "TEXT"),
            FrameType::Binary => write!(f, "BINARY"),
            FrameType::Close => write!(f, "CLOSE"),
            FrameType::Ping => write!(f, "PING"),
            FrameType::Pong => write!(f, "PONG"),
        }
    }
}

/// A captured WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub frame_type: FrameType,
    pub payload: String,
    pub timestamp: String,
    pub direction: Direction,
}

/// Direction of the message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Sent,
    Received,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Sent => write!(f, "→"),
            Direction::Received => write!(f, "←"),
        }
    }
}

/// Close frame details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

/// A WebSocket session capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub url: String,
    pub messages: Vec<Message>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub close_frame: Option<CloseFrame>,
}

impl Session {
    pub fn new(url: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            url: url.to_string(),
            messages: Vec::new(),
            started_at: now,
            ended_at: None,
            close_frame: None,
        }
    }

    pub fn add_message(&mut self, frame_type: FrameType, payload: String, direction: Direction) {
        self.messages.push(Message {
            frame_type,
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
            direction,
        });
    }

    #[allow(dead_code)]
    pub fn set_close(&mut self, code: u16, reason: String) {
        self.ended_at = Some(chrono::Utc::now().to_rfc3339());
        self.close_frame = Some(CloseFrame { code, reason });
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    #[allow(dead_code)]
    pub fn text_messages(&self) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|m| m.frame_type == FrameType::Text)
            .collect()
    }

    #[allow(dead_code)]
    pub fn binary_messages(&self) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|m| m.frame_type == FrameType::Binary)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("ws://example.com");
        assert_eq!(session.url, "ws://example.com");
        assert_eq!(session.message_count(), 0);
        assert!(!session.started_at.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut session = Session::new("ws://example.com");
        session.add_message(FrameType::Text, "hello".to_string(), Direction::Sent);
        assert_eq!(session.message_count(), 1);
        assert_eq!(session.text_messages().len(), 1);
    }

    #[test]
    fn test_frame_type_display() {
        assert_eq!(format!("{}", FrameType::Text), "TEXT");
        assert_eq!(format!("{}", FrameType::Binary), "BINARY");
        assert_eq!(format!("{}", FrameType::Close), "CLOSE");
        assert_eq!(format!("{}", FrameType::Ping), "PING");
        assert_eq!(format!("{}", FrameType::Pong), "PONG");
    }

    #[test]
    fn test_direction_display() {
        assert_eq!(format!("{}", Direction::Sent), "→");
        assert_eq!(format!("{}", Direction::Received), "←");
    }

    #[test]
    fn test_close_frame() {
        let mut session = Session::new("ws://example.com");
        session.add_message(FrameType::Text, "hello".to_string(), Direction::Received);
        session.set_close(1000, "Normal closure".to_string());
        assert!(session.ended_at.is_some());
        assert!(session.close_frame.is_some());
        assert_eq!(session.close_frame.as_ref().unwrap().code, 1000);
    }
}
