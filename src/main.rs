use clap::Parser;
use std::io::{self, Write};
use std::process::ExitCode;
use wa_ro::cli::{
    AuthCommands, ChatsCommands, Cli, Commands, GroupsCommands, MessagesCommands,
};
use wa_ro::models::WatchFilter;
use wa_ro::store::resolve_store;
use wa_ro::watcher::{Watcher, WatcherOptions};

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let store = resolve_store(cli.fixtures.as_deref(), cli.db.as_deref())?;

    match cli.command {
        Commands::Auth { subcommand } => match subcommand {
            AuthCommands::Pair { phone, qr: _ } => {
                let status = store.auth_status()?;
                println!("[wa-ro auth pair]");
                println!("Initiating read-only companion pairing flow...");
                println!("Session store: {}", status.store_path);
                println!("Protocol: WhatsApp Web companion link (Multi-Device)");
                println!("Safety Invariant: wa-ro is strictly READ-ONLY (no send paths compiled).");
                if let Some(ref num) = phone {
                    println!("Requested pairing code for phone: {num}");
                    println!("Pairing code: WA-RO-7788-9900 (stub)");
                } else {
                    println!("Scan QR code using WhatsApp -> Linked Devices -> Link a Device.");
                    println!("[QR Code stub placeholder: https://wa.me/qr/stub]");
                }
                println!("Session paired successfully (stub).");
            }
            AuthCommands::Status { json } => {
                let status = store.auth_status()?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    let state = if status.is_authenticated {
                        "Authenticated / Paired"
                    } else {
                        "Not Authenticated"
                    };
                    println!("WhatsApp Read-Only Session Status");
                    println!("  Status:          {state}");
                    println!(
                        "  Session User:    {}",
                        status.session_user.as_deref().unwrap_or("<none>")
                    );
                    println!("  Backend:         {}", status.backend);
                    println!("  Store Path:      {}", status.store_path);
                    println!(
                        "  Last Sync:       {}",
                        status
                            .last_sync
                            .map(|t| t.to_rfc3339())
                            .unwrap_or_else(|| "<never>".to_string())
                    );
                    println!("  Safety Policy:   STRICT READ-ONLY (no outbound send)");
                }
            }
        },
        Commands::Sync {
            once: _,
            follow: _,
            timeout: _,
        } => {
            let status = store.auth_status()?;
            let chats = store.list_chats(None, 1000)?;
            let groups = store.list_groups(None, 1000)?;
            let messages = store.list_messages(None, 5000)?;

            println!("[wa-ro sync]");
            println!("Synchronizing local message store...");
            println!("Backend: {}", status.backend);
            println!(
                "Store: {} chats, {} groups, {} messages cached locally.",
                chats.len(),
                groups.len(),
                messages.len()
            );
            println!("Sync state: Up-to-date (read-only mirror).");
        }
        Commands::Messages { subcommand } => match subcommand {
            MessagesCommands::List { chat, limit, json } => {
                let messages = store.list_messages(chat.as_deref(), limit)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&messages)?);
                } else if messages.is_empty() {
                    println!("No messages found.");
                } else {
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    for msg in &messages {
                        Watcher::emit_message(&mut handle, msg, false)?;
                    }
                }
            }
            MessagesCommands::Search {
                query,
                chat,
                limit,
                json,
            } => {
                let messages = store.search_messages(&query, chat.as_deref(), limit)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&messages)?);
                } else if messages.is_empty() {
                    println!("No messages matching '{query}' found.");
                } else {
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    for msg in &messages {
                        Watcher::emit_message(&mut handle, msg, false)?;
                    }
                }
            }
            MessagesCommands::Show { id, json } => {
                let msg_opt = store.get_message(&id)?;
                match msg_opt {
                    Some(msg) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&msg)?);
                        } else {
                            println!("Message ID:   {}", msg.id);
                            println!(
                                "Chat:         {} ({})",
                                msg.chat_name.as_deref().unwrap_or(""),
                                msg.chat_jid
                            );
                            println!(
                                "Sender:       {} ({})",
                                msg.sender_name.as_deref().unwrap_or(""),
                                msg.sender_jid
                            );
                            println!("Timestamp:    {}", msg.timestamp.to_rfc3339());
                            println!("From Me:      {}", msg.is_from_me);
                            if let Some(ref m) = msg.media_type {
                                println!("Media Type:   {m}");
                            }
                            if let Some(ref q) = msg.quoted_message_id {
                                println!("Quoted MSG:   {q}");
                            }
                            println!("----------------------------------------");
                            println!("{}", msg.text);
                        }
                    }
                    None => {
                        anyhow::bail!("Message with ID '{id}' not found");
                    }
                }
            }
        },
        Commands::Chats { subcommand } => match subcommand {
            ChatsCommands::List { query, limit, json } => {
                let chats = store.list_chats(query.as_deref(), limit)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&chats)?);
                } else if chats.is_empty() {
                    println!("No chats found.");
                } else {
                    let mut stdout = io::stdout().lock();
                    writeln!(
                        stdout,
                        "{:<36} {:<32} {:<6} {:<6} PREVIEW",
                        "JID", "NAME", "GROUP", "UNREAD"
                    )?;
                    writeln!(stdout, "{}", "-".repeat(100))?;
                    for c in &chats {
                        let name = c.name.as_deref().unwrap_or("<unnamed>");
                        let preview = c.last_message_preview.as_deref().unwrap_or("");
                        writeln!(
                            stdout,
                            "{:<36} {:<32} {:<6} {:<6} {}",
                            c.jid,
                            truncate(name, 30),
                            if c.is_group { "yes" } else { "no" },
                            c.unread_count,
                            truncate(preview, 40)
                        )?;
                    }
                }
            }
        },
        Commands::Groups { subcommand } => match subcommand {
            GroupsCommands::List { query, limit, json } => {
                let groups = store.list_groups(query.as_deref(), limit)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&groups)?);
                } else if groups.is_empty() {
                    println!("No groups found.");
                } else {
                    let mut stdout = io::stdout().lock();
                    writeln!(
                        stdout,
                        "{:<32} {:<32} {:<6} TOPIC",
                        "JID", "NAME", "MEMBERS"
                    )?;
                    writeln!(stdout, "{}", "-".repeat(100))?;
                    for g in &groups {
                        let topic = g.topic.as_deref().unwrap_or("");
                        writeln!(
                            stdout,
                            "{:<32} {:<32} {:<6} {}",
                            g.jid,
                            truncate(&g.name, 30),
                            g.participant_count,
                            truncate(topic, 40)
                        )?;
                    }
                }
            }
        },
        Commands::Watch {
            group,
            query,
            json,
            follow,
            interval_ms,
            limit,
        } => {
            let filter = WatchFilter {
                group,
                query,
                since: None,
                limit,
            };

            let options = WatcherOptions {
                filter,
                json,
                follow,
                interval_ms,
            };

            let watcher = Watcher::new(&*store, options);
            let stdout = io::stdout();
            let handle = stdout.lock();
            watcher.run(handle)?;
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        truncated.push_str("...");
        truncated
    }
}
