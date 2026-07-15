//! WSForge — Comprehensive WebSocket CLI and Protocol Analyzer
//!
//! A Rust CLI tool for analyzing, debugging, and validating WebSocket connections.
#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod client;
mod protocol;
mod replay;
mod types;
mod validate;

use clap::Parser;
use cli::{Cli, Commands};
use log::{debug, info};

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Connect {
            url,
            message,
            binary,
            timeout,
            output: _,
        } => {
            let mut client = client::WsClient::new(&url, std::time::Duration::from_secs(timeout));

            if let Some(msg) = message {
                match client.connect_and_send(&url, &msg, binary).await {
                    Ok(()) => {
                        info!("Connection completed successfully");
                        // Print session summary
                        let session = client.session();
                        println!("Session: {}", session.url);
                        println!("Messages: {}", session.message_count());
                        for msg in session.messages.iter() {
                            println!(
                                "  {} [{}] {}",
                                msg.direction,
                                msg.frame_type,
                                msg.payload.chars().take(200).collect::<String>()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                info!("No message provided. Use --message to send a message.");
                println!("Connected to {}", url);
                println!("Use --message to send a message after connecting.");
            }
        }

        Commands::Replay {
            input,
            url,
            speed,
            repeat,
        } => {
            let session_data = cli::read_session(&input).unwrap_or_else(|e| {
                eprintln!("Error reading session: {}", e);
                std::process::exit(1);
            });

            match replay::ReplaySession::from_json(&session_data) {
                Ok(session) => {
                    let replay = replay::ReplaySession::new(session, speed, repeat);
                    println!("{}", replay.summary());
                    println!();
                    println!("URL: {}", url);
                    println!("Speed: {:.1}x, Repeat: {} times", speed, repeat);
                    println!("Messages: {}", replay.session.messages.len());
                }
                Err(e) => {
                    eprintln!("Error parsing session: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Validate {
            schema,
            messages,
            format,
        } => {
            let schema_data = cli::read_session(&schema).unwrap_or_else(|e| {
                eprintln!("Error reading schema: {}", e);
                std::process::exit(1);
            });

            let schema = validate::parse_schema(&schema_data).unwrap_or_else(|e| {
                eprintln!("Error parsing schema: {}", e);
                std::process::exit(1);
            });

            let messages_data = cli::read_session(&messages).unwrap_or_else(|e| {
                eprintln!("Error reading messages: {}", e);
                std::process::exit(1);
            });

            let mut results = Vec::new();
            let mut total_valid = 0;
            let mut total_invalid = 0;

            for (i, line) in messages_data.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let result = validate::validate(line, &schema);
                let valid = result.valid;
                results.push((i + 1, result));
                if valid {
                    total_valid += 1;
                } else {
                    total_invalid += 1;
                }
            }

            match format.as_str() {
                "json" => {
                    let json_output = serde_json::json!({
                        "total": results.len(),
                        "valid": total_valid,
                        "invalid": total_invalid,
                        "results": results.iter().map(|(i, r)| {
                            serde_json::json!({
                                "line": i,
                                "valid": r.valid,
                                "errors": r.errors,
                                "warnings": r.warnings,
                            })
                        }).collect::<Vec<_>>(),
                    });
                    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
                }
                "markdown" => {
                    println!("# Validation Report\n");
                    println!("| Line | Valid | Errors | Warnings |");
                    println!("|------|-------|--------|----------|");
                    for (i, r) in &results {
                        let status = if r.valid { "✓" } else { "✗" };
                        println!(
                            "| {} | {} | {} | {} |",
                            i,
                            status,
                            r.errors.len(),
                            r.warnings.len()
                        );
                    }
                    println!();
                    println!(
                        "**Total:** {} | **Valid:** {} | **Invalid:** {}",
                        results.len(),
                        total_valid,
                        total_invalid
                    );
                }
                _ => {
                    println!("Validation Report");
                    println!("=================");
                    println!(
                        "Total: {} | Valid: {} | Invalid: {}",
                        results.len(),
                        total_valid,
                        total_invalid
                    );
                    println!();
                    for (i, r) in &results {
                        if r.valid {
                            println!("  Line {}: ✓ Valid", i);
                        } else {
                            println!("  Line {}: ✗ Invalid", i);
                            for err in &r.errors {
                                println!("    - {}", err);
                            }
                            for warn in &r.warnings {
                                println!("    ⚠ {}", warn);
                            }
                        }
                    }
                }
            }
        }

        Commands::Types {
            input,
            language,
            name,
        } => {
            let samples_data = cli::read_session(&input).unwrap_or_else(|e| {
                eprintln!("Error reading samples: {}", e);
                std::process::exit(1);
            });

            let samples: Vec<String> = samples_data
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .collect();

            let gen = types::TypeGenerator::new(&name);

            match language.as_str() {
                "typescript" => {
                    let types = gen.generate_typescript(&samples);
                    for t in &types {
                        println!("// {} definition", t.name);
                        println!("{};", t.definition);
                    }
                }
                "python" => {
                    let types = gen.generate_python(&samples);
                    for t in &types {
                        for imp in &t.imports {
                            println!("{}", imp);
                        }
                        println!();
                        println!("{}", t.definition);
                    }
                }
                _ => {
                    eprintln!(
                        "Unsupported language: {}. Use 'typescript' or 'python'.",
                        language
                    );
                    std::process::exit(1);
                }
            }
        }

        Commands::Inspect { url } => {
            println!("WebSocket URL: {}", url);
            println!(
                "Protocol: {}",
                if url.starts_with("wss://") {
                    "wss (TLS)"
                } else {
                    "ws (plain)"
                }
            );

            // Parse the URL
            if let Ok(parsed) = url::Url::parse(&url) {
                println!("Host: {}", parsed.host_str().unwrap_or("unknown"));
                println!("Port: {}", parsed.port_or_known_default().unwrap_or(80));
                println!("Path: {}", parsed.path());
            }

            println!("\nWebSocket Upgrade Headers:");
            println!("  Upgrade: websocket");
            println!("  Connection: Upgrade");
            println!("  Sec-WebSocket-Version: 13");
            println!("  Sec-WebSocket-Protocol: (negotiated)");
        }

        Commands::Format { input, format } => {
            let session_data = cli::read_session(&input).unwrap_or_else(|e| {
                eprintln!("Error reading session: {}", e);
                std::process::exit(1);
            });

            match replay::ReplaySession::from_json(&session_data) {
                Ok(session) => {
                    let replay = replay::ReplaySession::new(session, 1.0, 1);
                    match format.as_str() {
                        "json" => match replay.to_json() {
                            Ok(json) => println!("{}", json),
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        },
                        "markdown" => {
                            println!("{}", replay.to_markdown());
                        }
                        _ => {
                            println!("{}", replay.summary());
                            println!();
                            for msg in replay.session.messages.iter() {
                                println!(
                                    "{} [{}] {}",
                                    msg.direction,
                                    msg.frame_type,
                                    msg.payload.chars().take(200).collect::<String>()
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing session: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Output format for CLI commands
enum OutputFormat {
    Text,
    Json,
    Markdown,
}

impl cli::Cli {
    fn output_format(&self, format: &str) -> OutputFormat {
        match format {
            "json" => OutputFormat::Json,
            "markdown" => OutputFormat::Markdown,
            _ => OutputFormat::Text,
        }
    }
}
