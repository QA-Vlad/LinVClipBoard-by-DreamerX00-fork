use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use shared::config::AppConfig;
use shared::ipc::send_request;
use shared::models::{ContentType, IpcRequest, IpcResponse};

#[derive(Parser)]
#[command(name = "clipctl")]
#[command(about = "LinVClipBoard CLI — manage your clipboard history")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List clipboard history
    List {
        /// Maximum number of items to show
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
        /// Offset for pagination
        #[arg(short, long, default_value_t = 0)]
        offset: u32,
    },
    /// Search clipboard history
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },
    /// Paste an item back to clipboard
    Paste {
        /// Item ID
        id: String,
    },
    /// Toggle pin on an item
    Pin {
        /// Item ID
        id: String,
    },
    /// Delete an item
    Delete {
        /// Item ID
        id: String,
    },
    /// Clear all non-pinned items
    Clear,
    /// Show daemon status
    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let socket_path = AppConfig::socket_path();

    if !socket_path.exists() {
        eprintln!("{}", "Error: clipd daemon is not running.".red().bold());
        eprintln!("Start it with: {}", "clipd".cyan());
        eprintln!("Or enable the service: {}", "systemctl --user enable --now clipd".cyan());
        std::process::exit(1);
    }

    let request = match &cli.command {
        Commands::List { limit, offset } => IpcRequest::List {
            offset: *offset,
            limit: *limit,
        },
        Commands::Search { query, limit } => IpcRequest::Search {
            query: query.clone(),
            limit: *limit,
        },
        Commands::Paste { id } => IpcRequest::Paste { id: id.clone() },
        Commands::Pin { id } => IpcRequest::TogglePin { id: id.clone() },
        Commands::Delete { id } => IpcRequest::Delete { id: id.clone() },
        Commands::Clear => IpcRequest::Clear,
        Commands::Status => IpcRequest::Status,
    };

    match send_request(&socket_path, &request).await {
        Ok(response) => print_response(&cli.command, response),
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn print_response(_command: &Commands, response: IpcResponse) {
    match response {
        IpcResponse::Items { items, total } => {
            if items.is_empty() {
                println!("{}", "No items found.".dimmed());
                return;
            }

            println!(
                "{} ({} total)\n",
                "📋 Clipboard History".bold().cyan(),
                total.to_string().yellow()
            );

            for item in &items {
                let pin_icon = if item.pinned { "📌 " } else { "   " };
                let type_icon = match item.content_type {
                    ContentType::PlainText => "📝",
                    ContentType::Html => "🌐",
                    ContentType::Image => "🖼️",
                    ContentType::RichText => "📄",
                };

                let time = format_time(&item.created_at);
                let id_short = &item.id[..8];

                // Preview: truncate to 60 chars
                let preview = if item.preview_text.len() > 60 {
                    format!("{}...", &item.preview_text[..57])
                } else {
                    item.preview_text.clone()
                };
                // Replace newlines for display
                let preview = preview.replace('\n', "↵ ");

                println!(
                    "{}{} {} {} {}",
                    pin_icon,
                    id_short.dimmed(),
                    type_icon,
                    preview,
                    time.dimmed()
                );
            }

            println!(
                "\n{} Use {} to paste an item",
                "💡".dimmed(),
                "clipctl paste <id>".cyan()
            );
        }

        IpcResponse::Item(item) => {
            let pin_status = if item.pinned { "📌 Pinned" } else { "Unpinned" };
            println!("{} {}", "Item:".bold(), item.id.cyan());
            println!("  Type: {:?}", item.content_type);
            println!("  Status: {}", pin_status);
            println!("  Size: {} bytes", item.size_bytes);
            println!("  Created: {}", item.created_at.format("%Y-%m-%d %H:%M:%S"));
            if item.content_type != ContentType::Image {
                let preview = if item.preview_text.len() > 200 {
                    format!("{}...", &item.preview_text[..197])
                } else {
                    item.preview_text.clone()
                };
                println!("  Preview: {}", preview);
            }
        }

        IpcResponse::Ok { message } => {
            println!("{} {}", "✅".green(), message);
        }

        IpcResponse::Error { message } => {
            eprintln!("{} {}", "❌".red(), message);
            std::process::exit(1);
        }

        IpcResponse::Status {
            uptime_secs,
            total_items,
            db_size_bytes,
        } => {
            println!("{}", "📊 LinVClipBoard Status".bold().cyan());
            println!("  Uptime:      {}", format_duration(uptime_secs).yellow());
            println!("  Total items: {}", total_items.to_string().yellow());
            println!("  DB size:     {}", format_bytes(db_size_bytes).yellow());
        }
    }
}

fn format_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*dt);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
