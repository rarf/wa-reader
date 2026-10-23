use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an individual or group conversation in WhatsApp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chat {
    pub jid: String,
    pub name: Option<String>,
    pub is_group: bool,
    pub unread_count: u32,
    pub last_message_timestamp: Option<DateTime<Utc>>,
    pub last_message_preview: Option<String>,
}

/// Represents metadata for a WhatsApp group chat.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Group {
    pub jid: String,
    pub name: String,
    pub topic: Option<String>,
    pub owner_jid: Option<String>,
    pub participant_count: usize,
    pub created_at: Option<DateTime<Utc>>,
}

/// Represents an incoming or stored WhatsApp message.
/// Note: There are NO outbound send structures in this crate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    pub chat_jid: String,
    pub chat_name: Option<String>,
    pub sender_jid: String,
    pub sender_name: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub is_from_me: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted_message_id: Option<String>,
}

/// Represents the authentication and synchronization state of the companion session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub session_user: Option<String>,
    pub store_path: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub backend: String,
}

/// Filter criteria for monitoring/watching messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchFilter {
    /// Group JID or substring match on group name (case-insensitive)
    pub group: Option<String>,
    /// Keyword search in message body (case-insensitive)
    pub query: Option<String>,
    /// Only messages strictly after this timestamp
    pub since: Option<DateTime<Utc>>,
    /// Maximum number of messages/events to produce
    pub limit: Option<usize>,
}

impl WatchFilter {
    /// Checks whether a given message matches this filter's criteria.
    pub fn matches(&self, msg: &Message) -> bool {
        if let Some(ref group_pat) = self.group {
            let pat = group_pat.to_lowercase();
            let jid_matches = msg.chat_jid.to_lowercase().contains(&pat);
            let name_matches = msg
                .chat_name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&pat))
                .unwrap_or(false);
            if !jid_matches && !name_matches {
                return false;
            }
        }

        if let Some(ref q) = self.query {
            let q_lower = q.to_lowercase();
            let text_matches = msg.text.to_lowercase().contains(&q_lower);
            let sender_matches = msg
                .sender_name
                .as_ref()
                .map(|s| s.to_lowercase().contains(&q_lower))
                .unwrap_or(false);
            if !text_matches && !sender_matches {
                return false;
            }
        }

        if let Some(since_ts) = self.since
            && msg.timestamp <= since_ts
        {
            return false;
        }

        true
    }
}
