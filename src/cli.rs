use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wsforge")]
#[command(about = "Comprehensive WebSocket CLI and protocol analyzer")]
#[command(version)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Connect to a WebSocket server
    Connect {
        /// WebSocket URL to connect to
        #[arg(short, long)]
        url: String,

        /// Message to send after connecting (text)
        #[arg(short, long)]
        message: Option<String>,

        /// Send as binary data
        #[arg(long, default_value_t = false)]
        binary: bool,

        /// Timeout in seconds (0 = no timeout)
        #[arg(long, default_value_t = 30)]
        timeout: u64,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        output: String,
    },

    /// Replay a captured session
    Replay {
        /// Path to the captured session file (JSON)
        #[arg(short, long)]
        input: PathBuf,

        /// WebSocket URL to replay against
        #[arg(short, long)]
        url: String,

        /// Speed multiplier (1 = real-time)
        #[arg(long, default_value_t = 1.0)]
        speed: f64,

        /// Repeat the session N times
        #[arg(long, default_value_t = 1)]
        repeat: u32,
    },

    /// Validate messages against a JSON schema
    Validate {
        /// Path to the JSON schema file
        #[arg(short, long)]
        schema: PathBuf,

        /// Path to the messages file (one JSON per line)
        #[arg(short, long)]
        messages: PathBuf,

        /// Output validation report (text, json, markdown)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Generate type definitions from message samples
    Types {
        /// Input file with sample messages (one JSON per line)
        #[arg(short, long)]
        input: PathBuf,

        /// Output language (typescript, python)
        #[arg(long, default_value = "typescript")]
        language: String,

        /// Output type name
        #[arg(long, default_value = "Message")]
        name: String,
    },

    /// Inspect a WebSocket handshake
    Inspect {
        /// WebSocket URL to inspect
        #[arg(short, long)]
        url: String,
    },

    /// Format and beautify a captured session
    Format {
        /// Input session file
        #[arg(short, long)]
        input: PathBuf,

        /// Output format (text, json, markdown)
        #[arg(long, default_value = "text")]
        format: String,
    },
}

/// Read session data from a file or stdin
pub fn read_session(input: &PathBuf) -> Result<String, String> {
    if input.as_os_str().is_empty() || input.as_os_str() == "-" {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| e.to_string())?;
        Ok(buffer)
    } else {
        fs::read_to_string(input).map_err(|e| format!("Failed to read {}: {}", input.display(), e))
    }
}
