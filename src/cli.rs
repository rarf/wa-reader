use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "wa-ro")]
#[command(author = "wa-ro contributors")]
#[command(version)]
#[command(
    about = "A read-only WhatsApp CLI tool for humans and Chief of Staff / Grok Bot watchers",
    long_about = "wa-ro is a scriptable, strictly READ-ONLY WhatsApp CLI.\n\
                  It allows companion pairing, synchronization, listing chats/groups,\n\
                  full-text search of messages, and JSON-lines watchers for alerting.\n\
                  CRITICAL SAFETY INVARIANT: wa-ro contains NO 'send' subcommand and NO\n\
                  outbound messaging capabilities, preventing AI agents or automated bots\n\
                  from accidentally transmitting unwanted WhatsApp messages."
)]
pub struct Cli {
    /// Path to synthetic fixture JSON file (overrides default bundled fixture)
    #[arg(long, global = true)]
    pub fixtures: Option<PathBuf>,

    /// Path to SQLite database file
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// Companion authentication and session status (read-only)
    Auth {
        #[command(subcommand)]
        subcommand: AuthCommands,
    },
    /// Synchronize local message cache from WhatsApp session
    Sync {
        /// Perform a single sync cycle and exit
        #[arg(long)]
        once: bool,
        /// Continuously follow / stream incoming updates
        #[arg(short, long)]
        follow: bool,
        /// Sync timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    /// Inspect or search WhatsApp messages
    Messages {
        #[command(subcommand)]
        subcommand: MessagesCommands,
    },
    /// List individual and group chats
    Chats {
        #[command(subcommand)]
        subcommand: ChatsCommands,
    },
    /// List WhatsApp groups
    Groups {
        #[command(subcommand)]
        subcommand: GroupsCommands,
    },
    /// Stream or filter incoming messages matching group or query criteria (JSON lines)
    Watch {
        /// Filter by group name or group JID (case-insensitive substring)
        #[arg(long, short = 'g')]
        group: Option<String>,

        /// Filter messages by text or sender query
        #[arg(long, short = 'q')]
        query: Option<String>,

        /// Output as newline-delimited JSON (JSON lines)
        #[arg(long)]
        json: bool,

        /// Follow / stream updates continuously
        #[arg(long, short = 'f')]
        follow: bool,

        /// Polling interval in milliseconds when following
        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,

        /// Limit the number of events/messages emitted
        #[arg(long, short = 'n')]
        limit: Option<usize>,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum AuthCommands {
    /// Pair as a WhatsApp Web companion device (stub flow)
    Pair {
        /// Pair using phone number pairing code instead of QR
        #[arg(long)]
        phone: Option<String>,
        /// Print terminal QR code
        #[arg(long)]
        qr: bool,
    },
    /// Show companion session authentication status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum MessagesCommands {
    /// List recent messages
    List {
        /// Filter by chat name or JID
        #[arg(long, short = 'c')]
        chat: Option<String>,
        /// Maximum number of messages
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Search message content and senders
    Search {
        /// Search query string
        #[arg()]
        query: String,
        /// Filter within chat name or JID
        #[arg(long, short = 'c')]
        chat: Option<String>,
        /// Maximum number of messages
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show a specific message by ID
    Show {
        /// Message ID to display
        #[arg()]
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum ChatsCommands {
    /// List conversation chats
    List {
        /// Filter by chat name or JID
        #[arg(long, short = 'q')]
        query: Option<String>,
        /// Maximum number of chats
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum GroupsCommands {
    /// List WhatsApp groups
    List {
        /// Filter by group name, topic or JID
        #[arg(long, short = 'q')]
        query: Option<String>,
        /// Maximum number of groups
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
