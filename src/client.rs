//! WebSocket client functionality
//!
//! Connects to WebSocket servers, sends/receives messages, and captures sessions.

use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use log::{debug, info, warn};
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::protocol::{Direction, FrameType, Session};

/// A WebSocket client that can connect, send, and receive messages
pub struct WsClient {
    session: Session,
    timeout: Duration,
}

impl WsClient {
    pub fn new(url: &str, timeout: Duration) -> Self {
        Self {
            session: Session::new(url),
            timeout,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    /// Connect to a WebSocket server and optionally send a message
    pub async fn connect_and_send(
        &mut self,
        url: &str,
        message: &str,
        is_binary: bool,
    ) -> Result<(), String> {
        let (mut socket, _) = connect_async(url)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        debug!("Connected to {}", url);

        let ws_msg = if is_binary {
            Message::binary(message.as_bytes().to_vec())
        } else {
            Message::text(message.to_string())
        };

        socket
            .send(ws_msg.clone())
            .await
            .map_err(|e| format!("Send failed: {}", e))?;
        self.session
            .add_message(ws_frame_type(&ws_msg), message.to_string(), Direction::Sent);

        info!("Sent: {}", message);

        // Wait for response with timeout
        tokio::time::timeout(self.timeout, async { socket.next().await })
            .await
            .map_err(|_| format!("Timeout after {}s", self.timeout.as_secs()))?;
        Ok(())
    }

    /// Capture a full session: connect, send message, receive all responses until close
    pub async fn capture_session(
        &mut self,
        url: &str,
        send_message: &str,
        is_binary: bool,
    ) -> Result<Session, String> {
        let (mut socket, _) = connect_async(url)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Send initial message
        let ws_msg = if is_binary {
            Message::binary(send_message.as_bytes().to_vec())
        } else {
            Message::text(send_message.to_string())
        };
        socket
            .send(ws_msg.clone())
            .await
            .map_err(|e| format!("Send failed: {}", e))?;
        self.session.add_message(
            ws_frame_type(&ws_msg),
            send_message.to_string(),
            Direction::Sent,
        );

        // Receive messages until close or timeout
        loop {
            match tokio::time::timeout(self.timeout, async { socket.next().await }).await {
                Ok(Some(Ok(frame))) => {
                    let text = msg_to_string(&frame);
                    let frame_type = ws_frame_type(&frame);

                    self.session
                        .add_message(frame_type.clone(), text.clone(), Direction::Received);
                    debug!(
                        "Received {} message: {}",
                        frame_type,
                        text.chars().take(200).collect::<String>()
                    );

                    if frame.is_close() {
                        break;
                    }
                }
                Ok(Some(Err(e))) => {
                    warn!("Receive error: {}", e);
                    break;
                }
                Ok(None) => {
                    debug!("Stream ended");
                    break;
                }
                Err(_) => {
                    debug!("Timeout, ending session");
                    break;
                }
            }
        }

        Ok(self.session.clone())
    }
}

fn ws_frame_type(msg: &tungstenite::Message) -> FrameType {
    if msg.is_text() {
        FrameType::Text
    } else if msg.is_binary() {
        FrameType::Binary
    } else if msg.is_close() {
        FrameType::Close
    } else if msg.is_ping() {
        FrameType::Ping
    } else if msg.is_pong() {
        FrameType::Pong
    } else {
        FrameType::Text
    }
}

fn msg_to_string(msg: &tungstenite::Message) -> String {
    if msg.is_text() {
        msg.to_string()
    } else if msg.is_binary() {
        format!("[Binary: {} bytes]", msg.len())
    } else if msg.is_close() {
        "[Close]".to_string()
    } else if msg.is_ping() {
        format!("[Ping: {} bytes]", msg.len())
    } else if msg.is_pong() {
        format!("[Pong: {} bytes]", msg.len())
    } else {
        "[Unknown]".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WsClient::new("ws://example.com", Duration::from_secs(10));
        assert_eq!(client.session().url, "ws://example.com");
        assert_eq!(client.session().message_count(), 0);
    }
}
