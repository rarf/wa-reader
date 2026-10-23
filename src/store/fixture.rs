use crate::models::{AuthStatus, Chat, Group, Message};
use crate::store::{ReadOnlyStore, StoreError};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DEFAULT_FIXTURE_JSON: &str = include_str!("../../fixtures/sample_store.json");

/// Raw serialized container for fixture data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureData {
    pub auth: AuthStatus,
    #[serde(default)]
    pub chats: Vec<Chat>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

/// A read-only store powered by synthetic offline fixtures.
#[derive(Debug, Clone)]
pub struct FixtureStore {
    data: FixtureData,
    source_name: String,
}

impl FixtureStore {
    /// Create a FixtureStore from bundled synthetic fixture data.
    pub fn bundled() -> Result<Self, StoreError> {
        Self::from_json_str(DEFAULT_FIXTURE_JSON, "bundled:sample_store.json")
    }

    /// Create a FixtureStore from a JSON string.
    pub fn from_json_str(json: &str, source: &str) -> Result<Self, StoreError> {
        let data: FixtureData = serde_json::from_str(json)?;
        Ok(Self {
            data,
            source_name: source.to_string(),
        })
    }

    /// Load a FixtureStore from a filesystem path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path_ref = path.as_ref();
        let contents = std::fs::read_to_string(path_ref)?;
        Self::from_json_str(&contents, &path_ref.display().to_string())
    }

    /// Get the source identifier.
    pub fn source_name(&self) -> &str {
        &self.source_name
    }
}

impl Default for FixtureStore {
    fn default() -> Self {
        Self::bundled().expect("bundled fixture JSON must be valid")
    }
}

impl ReadOnlyStore for FixtureStore {
    fn auth_status(&self) -> Result<AuthStatus, StoreError> {
        let mut status = self.data.auth.clone();
        status.backend = format!("fixture ({})", self.source_name);
        Ok(status)
    }

    fn list_chats(&self, query: Option<&str>, limit: usize) -> Result<Vec<Chat>, StoreError> {
        let q_opt = query.map(|s| s.to_lowercase());
        let filtered: Vec<Chat> = self
            .data
            .chats
            .iter()
            .filter(|chat| {
                if let Some(ref q) = q_opt {
                    let jid_match = chat.jid.to_lowercase().contains(q);
                    let name_match = chat
                        .name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(q))
                        .unwrap_or(false);
                    jid_match || name_match
                } else {
                    true
                }
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }

    fn list_groups(&self, query: Option<&str>, limit: usize) -> Result<Vec<Group>, StoreError> {
        let q_opt = query.map(|s| s.to_lowercase());
        let filtered: Vec<Group> = self
            .data
            .groups
            .iter()
            .filter(|group| {
                if let Some(ref q) = q_opt {
                    let jid_match = group.jid.to_lowercase().contains(q);
                    let name_match = group.name.to_lowercase().contains(q);
                    let topic_match = group
                        .topic
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(q))
                        .unwrap_or(false);
                    jid_match || name_match || topic_match
                } else {
                    true
                }
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }

    fn list_messages(&self, chat: Option<&str>, limit: usize) -> Result<Vec<Message>, StoreError> {
        let chat_pat = chat.map(|s| s.to_lowercase());
        let mut messages: Vec<Message> = self
            .data
            .messages
            .iter()
            .filter(|msg| {
                if let Some(ref pat) = chat_pat {
                    let jid_match = msg.chat_jid.to_lowercase().contains(pat);
                    let name_match = msg
                        .chat_name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(pat))
                        .unwrap_or(false);
                    jid_match || name_match
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort by timestamp descending (most recent first) or ascending?
        // In CLI message listing, recent messages first is customary, or chronological.
        // Let's sort chronological (ascending) for chat context or allow limit.
        messages.sort_by_key(|m| m.timestamp);
        if messages.len() > limit {
            messages = messages.split_off(messages.len() - limit);
        }
        Ok(messages)
    }

    fn search_messages(
        &self,
        query: &str,
        chat: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Message>, StoreError> {
        let q = query.to_lowercase();
        let chat_pat = chat.map(|s| s.to_lowercase());

        let mut results: Vec<Message> = self
            .data
            .messages
            .iter()
            .filter(|msg| {
                if let Some(ref pat) = chat_pat {
                    let jid_match = msg.chat_jid.to_lowercase().contains(pat);
                    let name_match = msg
                        .chat_name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(pat))
                        .unwrap_or(false);
                    if !jid_match && !name_match {
                        return false;
                    }
                }

                let text_match = msg.text.to_lowercase().contains(&q);
                let sender_match = msg
                    .sender_name
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&q))
                    .unwrap_or(false);
                let chat_match = msg
                    .chat_name
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&q))
                    .unwrap_or(false);
                text_match || sender_match || chat_match
            })
            .cloned()
            .collect();

        results.sort_by_key(|m| m.timestamp);
        if results.len() > limit {
            results = results.split_off(results.len() - limit);
        }
        Ok(results)
    }

    fn get_message(&self, id: &str) -> Result<Option<Message>, StoreError> {
        let target = id.to_lowercase();
        let found = self
            .data
            .messages
            .iter()
            .find(|m| m.id.to_lowercase() == target)
            .cloned();
        Ok(found)
    }
}
