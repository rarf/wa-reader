#[cfg(feature = "sqlite")]
use crate::models::{AuthStatus, Chat, Group, Message};
#[cfg(feature = "sqlite")]
use crate::store::{ReadOnlyStore, StoreError};
#[cfg(feature = "sqlite")]
use chrono::{DateTime, Utc};
#[cfg(feature = "sqlite")]
use rusqlite::{params, Connection, OpenFlags};
#[cfg(feature = "sqlite")]
use std::path::{Path, PathBuf};
#[cfg(feature = "sqlite")]
use std::sync::Mutex;

#[cfg(feature = "sqlite")]
pub struct SqliteStore {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

#[cfg(feature = "sqlite")]
impl SqliteStore {
    /// Open an existing SQLite database in read-only mode, or create schema if open_read_write is true.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path_buf = path.as_ref().to_path_buf();
        // Try opening read-only first
        let conn = match Connection::open_with_flags(&path_buf, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(c) => c,
            Err(_) => {
                // If it doesn't exist yet, create with schema
                if let Some(parent) = path_buf.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let c = Connection::open(&path_buf)
                    .map_err(|e| StoreError::Database(e.to_string()))?;
                Self::init_schema(&c)?;
                c
            }
        };

        Ok(Self {
            conn: Mutex::new(conn),
            db_path: path_buf,
        })
    }

    /// Initialize tables if creating a new database.
    pub fn init_schema(conn: &Connection) -> Result<(), StoreError> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS chats (
                jid TEXT PRIMARY KEY,
                name TEXT,
                is_group INTEGER NOT NULL DEFAULT 0,
                unread_count INTEGER NOT NULL DEFAULT 0,
                last_message_timestamp TEXT,
                last_message_preview TEXT
            );
            CREATE TABLE IF NOT EXISTS groups_meta (
                jid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                topic TEXT,
                owner_jid TEXT,
                participant_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                chat_jid TEXT NOT NULL,
                chat_name TEXT,
                sender_jid TEXT NOT NULL,
                sender_name TEXT,
                timestamp TEXT NOT NULL,
                text TEXT NOT NULL,
                is_from_me INTEGER NOT NULL DEFAULT 0,
                media_type TEXT,
                quoted_message_id TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_messages_chat_jid ON messages(chat_jid);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);
            ",
        )
        .map_err(|e| StoreError::Database(e.to_string()))?;
        Ok(())
    }

    /// Helper to seed the sqlite database from fixture data (useful for testing/demo).
    pub fn seed_from_fixture(&self, fixture: &crate::store::fixture::FixtureStore) -> Result<(), StoreError> {
        let conn = self.conn.lock().unwrap();
        Self::init_schema(&conn)?;

        let auth = fixture.auth_status()?;
        let _ = conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('session_user', ?1), ('last_sync', ?2)",
            params![
                auth.session_user.unwrap_or_default(),
                auth.last_sync.map(|t| t.to_rfc3339()).unwrap_or_default()
            ],
        );

        for chat in fixture.list_chats(None, 1000)? {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO chats (jid, name, is_group, unread_count, last_message_timestamp, last_message_preview)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    chat.jid,
                    chat.name,
                    chat.is_group as i32,
                    chat.unread_count,
                    chat.last_message_timestamp.map(|t| t.to_rfc3339()),
                    chat.last_message_preview,
                ],
            );
        }

        for group in fixture.list_groups(None, 1000)? {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO groups_meta (jid, name, topic, owner_jid, participant_count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    group.jid,
                    group.name,
                    group.topic,
                    group.owner_jid,
                    group.participant_count as i64,
                    group.created_at.map(|t| t.to_rfc3339()),
                ],
            );
        }

        for msg in fixture.list_messages(None, 5000)? {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO messages (id, chat_jid, chat_name, sender_jid, sender_name, timestamp, text, is_from_me, media_type, quoted_message_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    msg.id,
                    msg.chat_jid,
                    msg.chat_name,
                    msg.sender_jid,
                    msg.sender_name,
                    msg.timestamp.to_rfc3339(),
                    msg.text,
                    msg.is_from_me as i32,
                    msg.media_type,
                    msg.quoted_message_id,
                ],
            );
        }

        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl ReadOnlyStore for SqliteStore {
    fn auth_status(&self) -> Result<AuthStatus, StoreError> {
        let conn = self.conn.lock().unwrap();
        let user: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'session_user'",
                [],
                |r| r.get(0),
            )
            .ok();
        let last_sync_str: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'last_sync'",
                [],
                |r| r.get(0),
            )
            .ok();
        let last_sync = last_sync_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));

        Ok(AuthStatus {
            is_authenticated: user.is_some(),
            session_user: user,
            store_path: self.db_path.display().to_string(),
            last_sync,
            backend: "sqlite".to_string(),
        })
    }

    fn list_chats(&self, query: Option<&str>, limit: usize) -> Result<Vec<Chat>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let q_like = query.map(|q| format!("%{q}%"));

        let mut chats = Vec::new();
        if let Some(ref q) = q_like {
            let sql = "SELECT jid, name, is_group, unread_count, last_message_timestamp, last_message_preview FROM chats WHERE (jid LIKE ?1 OR name LIKE ?1) ORDER BY last_message_timestamp DESC LIMIT ?2";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![q, limit as i64], parse_chat)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                chats.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        } else {
            let sql = "SELECT jid, name, is_group, unread_count, last_message_timestamp, last_message_preview FROM chats ORDER BY last_message_timestamp DESC LIMIT ?1";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![limit as i64], parse_chat)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                chats.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        }
        Ok(chats)
    }

    fn list_groups(&self, query: Option<&str>, limit: usize) -> Result<Vec<Group>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let q_like = query.map(|q| format!("%{q}%"));

        let mut groups = Vec::new();
        if let Some(ref q) = q_like {
            let sql = "SELECT jid, name, topic, owner_jid, participant_count, created_at FROM groups_meta WHERE (jid LIKE ?1 OR name LIKE ?1 OR topic LIKE ?1) ORDER BY name ASC LIMIT ?2";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![q, limit as i64], parse_group)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                groups.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        } else {
            let sql = "SELECT jid, name, topic, owner_jid, participant_count, created_at FROM groups_meta ORDER BY name ASC LIMIT ?1";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![limit as i64], parse_group)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                groups.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        }
        Ok(groups)
    }

    fn list_messages(&self, chat: Option<&str>, limit: usize) -> Result<Vec<Message>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let c_like = chat.map(|c| format!("%{c}%"));

        let mut msgs = Vec::new();
        if let Some(ref c) = c_like {
            let sql = "SELECT id, chat_jid, chat_name, sender_jid, sender_name, timestamp, text, is_from_me, media_type, quoted_message_id FROM messages WHERE (chat_jid LIKE ?1 OR chat_name LIKE ?1) ORDER BY timestamp ASC LIMIT ?2";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![c, limit as i64], parse_message)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                msgs.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        } else {
            let sql = "SELECT id, chat_jid, chat_name, sender_jid, sender_name, timestamp, text, is_from_me, media_type, quoted_message_id FROM messages ORDER BY timestamp ASC LIMIT ?1";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![limit as i64], parse_message)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                msgs.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        }
        Ok(msgs)
    }

    fn search_messages(
        &self,
        query: &str,
        chat: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Message>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let q_like = format!("%{query}%");
        let c_like = chat.map(|c| format!("%{c}%"));

        let mut msgs = Vec::new();
        if let Some(ref c) = c_like {
            let sql = "SELECT id, chat_jid, chat_name, sender_jid, sender_name, timestamp, text, is_from_me, media_type, quoted_message_id FROM messages WHERE (text LIKE ?1 OR sender_name LIKE ?1 OR chat_name LIKE ?1) AND (chat_jid LIKE ?2 OR chat_name LIKE ?2) ORDER BY timestamp ASC LIMIT ?3";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![q_like, c, limit as i64], parse_message)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                msgs.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        } else {
            let sql = "SELECT id, chat_jid, chat_name, sender_jid, sender_name, timestamp, text, is_from_me, media_type, quoted_message_id FROM messages WHERE (text LIKE ?1 OR sender_name LIKE ?1 OR chat_name LIKE ?1) ORDER BY timestamp ASC LIMIT ?2";
            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            let rows = stmt
                .query_map(params![q_like, limit as i64], parse_message)
                .map_err(|e| StoreError::Database(e.to_string()))?;
            for r in rows {
                msgs.push(r.map_err(|e| StoreError::Database(e.to_string()))?);
            }
        }
        Ok(msgs)
    }

    fn get_message(&self, id: &str) -> Result<Option<Message>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let sql = "SELECT id, chat_jid, chat_name, sender_jid, sender_name, timestamp, text, is_from_me, media_type, quoted_message_id FROM messages WHERE id = ?1";
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| StoreError::Database(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id], parse_message)
            .map_err(|e| StoreError::Database(e.to_string()))?;

        match rows.next() {
            Some(Ok(msg)) => Ok(Some(msg)),
            Some(Err(e)) => Err(StoreError::Database(e.to_string())),
            None => Ok(None),
        }
    }
}

#[cfg(feature = "sqlite")]
fn parse_chat(row: &rusqlite::Row) -> rusqlite::Result<Chat> {
    let jid: String = row.get(0)?;
    let name: Option<String> = row.get(1)?;
    let is_group: i32 = row.get(2)?;
    let unread_count: u32 = row.get(3)?;
    let ts_str: Option<String> = row.get(4)?;
    let preview: Option<String> = row.get(5)?;

    let last_message_timestamp = ts_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    Ok(Chat {
        jid,
        name,
        is_group: is_group != 0,
        unread_count,
        last_message_timestamp,
        last_message_preview: preview,
    })
}

#[cfg(feature = "sqlite")]
fn parse_group(row: &rusqlite::Row) -> rusqlite::Result<Group> {
    let jid: String = row.get(0)?;
    let name: String = row.get(1)?;
    let topic: Option<String> = row.get(2)?;
    let owner_jid: Option<String> = row.get(3)?;
    let participant_count: i64 = row.get(4)?;
    let ts_str: Option<String> = row.get(5)?;

    let created_at = ts_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    Ok(Group {
        jid,
        name,
        topic,
        owner_jid,
        participant_count: participant_count.max(0) as usize,
        created_at,
    })
}

#[cfg(feature = "sqlite")]
fn parse_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
    let id: String = row.get(0)?;
    let chat_jid: String = row.get(1)?;
    let chat_name: Option<String> = row.get(2)?;
    let sender_jid: String = row.get(3)?;
    let sender_name: Option<String> = row.get(4)?;
    let ts_str: String = row.get(5)?;
    let text: String = row.get(6)?;
    let is_from_me: i32 = row.get(7)?;
    let media_type: Option<String> = row.get(8)?;
    let quoted_message_id: Option<String> = row.get(9)?;

    let timestamp = DateTime::parse_from_rfc3339(&ts_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Message {
        id,
        chat_jid,
        chat_name,
        sender_jid,
        sender_name,
        timestamp,
        text,
        is_from_me: is_from_me != 0,
        media_type,
        quoted_message_id,
    })
}
