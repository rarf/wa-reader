# Milestone A — scaffold wa-ro (GO 2026-10-23)

Create a **Rust CLI** that is a **read-only** port of wacli (https://github.com/openclaw/wacli — Go + whatsmeow).

## Goal
Scriptable WhatsApp CLI for humans and Grok Bot (Chief of Staff): pair, sync, search/list messages, group watchers — **must not send messages**. Motivation: fear of an LLM sending the wrong WhatsApp message.

## Use cases
1. Watch a futsal/football group → alerts
2. Watch a school/children group → alerts for CoS
Optional later: bot template that "looks after the kids"

## Hard constraints
- **No `send` subcommand** in the public CLI. Prefer compile-time absence of send paths.
- No secrets in the repo. Session/store under XDG data dirs.
- Do **not** `gh repo create`, push to GitHub, merge, or deploy without explicit GO.
- Prefer solid scaffold + fixtures over a broken live WhatsApp client. Do not invent protocol details.

## Ship this milestone
1. Cargo crate/workspace with CLI (`clap`).
2. Subcommands: `auth` (pair/status stub), `sync` stub, `messages list|search|show`, `chats list`, `groups list`, `watch` (JSON lines for group JID/name filters).
3. Architecture note in README/docs: Rust whatsmeow equivalent research; if none, trait + stub/fixture adapter + phased plan (bridge to wacli store later).
4. Synthetic fixtures + tests so `watch` and `messages search` work offline.
5. LICENSE MIT, README (bilingual or pt-PT safety section OK).
6. `cargo test` and `cargo clippy -D warnings` clean.

## Success criteria
- Builds and tests pass
- `--help` shows only read-only commands (no send)
- README states CoS/watcher use cases + no-send invariant
- `watch --group <name-or-jid> --json` works against fixtures

Linear: RAR-11. When done, write a short RECEIPT.md with what was created and how to run tests.
