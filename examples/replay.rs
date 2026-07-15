//! Example: Replay a captured session
use wsforge::protocol::Session;
use wsforge::replay::ReplaySession;

fn main() {
    let session = Session::new("wss://echo.websocket.org");
    let replay = ReplaySession::new(session, 1.0, 1);
    println!("{}", replay.summary());
}
