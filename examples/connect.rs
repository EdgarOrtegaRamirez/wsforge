//! Example: Connect to a WebSocket server and send a message
use std::time::Duration;
use wsforge::client::WsClient;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut client = WsClient::new("wss://echo.websocket.org", Duration::from_secs(10));

    match client
        .connect_and_send("wss://echo.websocket.org", "Hello, WSForge!", false)
        .await
    {
        Ok(()) => {
            let session = client.session();
            println!("Messages received: {}", session.message_count());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
