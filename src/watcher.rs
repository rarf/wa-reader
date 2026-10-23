use crate::models::{Message, WatchFilter};
use crate::store::{ReadOnlyStore, StoreError};
use std::collections::HashSet;
use std::io::Write;
use std::time::Duration;

pub struct WatcherOptions {
    pub filter: WatchFilter,
    pub json: bool,
    pub follow: bool,
    pub interval_ms: u64,
}

impl Default for WatcherOptions {
    fn default() -> Self {
        Self {
            filter: WatchFilter::default(),
            json: false,
            follow: false,
            interval_ms: 1000,
        }
    }
}

pub struct Watcher<'a> {
    store: &'a dyn ReadOnlyStore,
    options: WatcherOptions,
}

impl<'a> Watcher<'a> {
    pub fn new(store: &'a dyn ReadOnlyStore, options: WatcherOptions) -> Self {
        Self { store, options }
    }

    /// Retrieve matching messages currently in store according to the filter.
    pub fn collect_matches(&self) -> Result<Vec<Message>, StoreError> {
        let messages = self.store.list_messages(None, 10_000)?;
        let mut matched: Vec<Message> = messages
            .into_iter()
            .filter(|m| self.options.filter.matches(m))
            .collect();

        matched.sort_by_key(|m| m.timestamp);

        if let Some(limit) = self.options.filter.limit
            && matched.len() > limit
        {
            matched = matched.split_off(matched.len() - limit);
        }
        Ok(matched)
    }

    /// Execute the watcher, writing JSON lines or text output to the provided writer.
    pub fn run<W: Write>(&self, mut out: W) -> Result<usize, StoreError> {
        let initial_matches = self.collect_matches()?;
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut emitted_count = 0;

        for msg in &initial_matches {
            seen_ids.insert(msg.id.clone());
            Self::emit_message(&mut out, msg, self.options.json)?;
            emitted_count += 1;
            if let Some(limit) = self.options.filter.limit
                && emitted_count >= limit
            {
                return Ok(emitted_count);
            }
        }

        if !self.options.follow {
            return Ok(emitted_count);
        }

        // Follow mode: poll for new messages
        let interval = Duration::from_millis(self.options.interval_ms);
        loop {
            std::thread::sleep(interval);
            let current_messages = self.store.list_messages(None, 1000)?;
            for msg in current_messages {
                if !seen_ids.contains(&msg.id) && self.options.filter.matches(&msg) {
                    seen_ids.insert(msg.id.clone());
                    Self::emit_message(&mut out, &msg, self.options.json)?;
                    emitted_count += 1;
                    if let Some(limit) = self.options.filter.limit
                        && emitted_count >= limit
                    {
                        return Ok(emitted_count);
                    }
                }
            }
        }
    }

    /// Helper to format and emit a single message.
    pub fn emit_message<W: Write>(mut out: W, msg: &Message, json: bool) -> Result<(), StoreError> {
        if json {
            let line = serde_json::to_string(msg)?;
            writeln!(out, "{line}")?;
        } else {
            let chat_display = msg.chat_name.as_deref().unwrap_or(&msg.chat_jid);
            let sender_display = msg.sender_name.as_deref().unwrap_or(&msg.sender_jid);
            writeln!(
                out,
                "[{}] [{}] {}: {}",
                msg.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                chat_display,
                sender_display,
                msg.text
            )?;
        }
        out.flush()?;
        Ok(())
    }
}
