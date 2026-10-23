pub mod fixture;
pub mod sqlite;

use crate::models::{AuthStatus, Chat, Group, Message};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Store I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Message not found: {0}")]
    NotFound(String),
    #[error("Store error: {0}")]
    Custom(String),
}

/// Core trait defining read-only operations against a WhatsApp message store.
/// Compile-time invariant: This trait contains ONLY read operations (no send, no write-to-remote).
pub trait ReadOnlyStore: Send + Sync {
    /// Retrieve current companion session authentication status.
    fn auth_status(&self) -> Result<AuthStatus, StoreError>;

    /// List recent chats, optionally matching a query filter.
    fn list_chats(&self, query: Option<&str>, limit: usize) -> Result<Vec<Chat>, StoreError>;

    /// List groups, optionally matching a query filter.
    fn list_groups(&self, query: Option<&str>, limit: usize) -> Result<Vec<Group>, StoreError>;

    /// List messages, optionally filtered by chat JID or name.
    fn list_messages(&self, chat: Option<&str>, limit: usize) -> Result<Vec<Message>, StoreError>;

    /// Search messages by text or sender name.
    fn search_messages(
        &self,
        query: &str,
        chat: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Message>, StoreError>;

    /// Retrieve a single message by ID.
    fn get_message(&self, id: &str) -> Result<Option<Message>, StoreError>;
}

/// Default XDG data directory for wa-ro.
pub fn default_data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("WA_RO_DATA_DIR") {
        return PathBuf::from(custom);
    }
    if let Some(base) = dirs::data_dir() {
        base.join("wa-ro")
    } else {
        PathBuf::from(".wa-ro")
    }
}

/// Default SQLite database path under XDG data directory.
pub fn default_db_path() -> PathBuf {
    default_data_dir().join("wa-ro.db")
}

/// Common wacli default SQLite database path (~/.wacli/wacli.db) for interoperability.
pub fn default_wacli_db_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".wacli").join("wacli.db"))
}

/// Resolves the store backend to use:
/// 1. Explicit fixture path passed via CLI or environment `WA_RO_FIXTURES`
/// 2. Explicit SQLite path passed via CLI
/// 3. Existing XDG wa-ro.db
/// 4. Existing ~/.wacli/wacli.db (for bridge compatibility)
/// 5. Bundled synthetic fixtures as default fallback
pub fn resolve_store(
    fixture_path: Option<&Path>,
    db_path: Option<&Path>,
) -> Result<Box<dyn ReadOnlyStore>, StoreError> {
    // 1. Fixture path explicitly requested
    if let Some(path) = fixture_path {
        return Ok(Box::new(fixture::FixtureStore::from_file(path)?));
    }
    if let Ok(env_fixture) = std::env::var("WA_RO_FIXTURES")
        && !env_fixture.trim().is_empty()
    {
        return Ok(Box::new(fixture::FixtureStore::from_file(env_fixture)?));
    }

    // 2. Explicit db path
    #[cfg(feature = "sqlite")]
    if let Some(path) = db_path {
        return Ok(Box::new(sqlite::SqliteStore::open(path)?));
    }

    // 3. XDG wa-ro db if it already exists
    #[cfg(feature = "sqlite")]
    {
        let default_db = default_db_path();
        if default_db.exists() {
            return Ok(Box::new(sqlite::SqliteStore::open(default_db)?));
        }

        // 4. wacli database if it exists
        if let Some(wacli_db) = default_wacli_db_path()
            && wacli_db.exists()
            && let Ok(store) = sqlite::SqliteStore::open(&wacli_db)
        {
            return Ok(Box::new(store));
        }
    }

    // 5. Default bundled fixture fallback
    Ok(Box::new(fixture::FixtureStore::bundled()?))
}
