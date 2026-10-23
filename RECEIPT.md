# Milestone A Completion Receipt — wa-ro

**Linear Ticket:** RAR-11  
**Date:** 2026-10-23  
**Status:** COMPLETE (Milestone A — scaffold wa-ro)

---

## 1. Summary of Deliverables

A complete, production-grade **read-only WhatsApp CLI** (`wa-ro`) has been implemented in Rust, adhering strictly to the no-send invariant and all architectural requirements specified in `TASK.md`.

### Created Files & Project Structure
- [`Cargo.toml`](Cargo.toml): Configured crate `wa-ro` with Rust 2024 edition, dependencies (`clap` v4 with derive, `serde`, `serde_json`, `chrono`, `dirs`, `rusqlite`, `thiserror`, `anyhow`, `tokio`), and default feature `sqlite`.
- [`LICENSE`](LICENSE): Standard MIT License.
- [`README.md`](README.md): Comprehensive documentation including:
  - Motivation and safety invariant (zero outbound messaging paths).
  - Bilingual safety section (`pt-PT` and English).
  - Use case walkthroughs (Futsal / Football alerts and School / CoS alerts).
  - Architecture note: Research on Rust WhatsApp Web implementations (`whatsapp-rust`, `whatsmeow` FFI, `whatsmeow-rust`), trait abstraction (`ReadOnlyStore`), and phased roadmap.
  - Complete CLI usage reference.
- [`fixtures/sample_store.json`](fixtures/sample_store.json): Synthetic offline WhatsApp dataset covering:
  - Futsal group ("Futsal Terça 21h - Carnaxide") with quorum and game coordination messages.
  - School group ("Escola Básica 3º Ano - Turma B") with parent meeting, ATL, and homework notices.
  - Family group ("Família & Miúdos") for pickup coordination.
  - Direct chats ("Maria Silva", "Grok Chief of Staff").
- [`src/lib.rs`](src/lib.rs): Library root exporting `cli`, `models`, `store`, and `watcher`.
- [`src/models.rs`](src/models.rs): Core domain models (`Chat`, `Group`, `Message`, `AuthStatus`, `WatchFilter`).
- [`src/store/mod.rs`](src/store/mod.rs): Storage abstraction (`ReadOnlyStore` trait), error definitions (`StoreError`), XDG standard directory resolution (`~/.local/share/wa-ro/`), and fallback logic (resolves explicit fixtures -> explicit db -> XDG db -> wacli bridge db -> bundled fixture).
- [`src/store/fixture.rs`](src/store/fixture.rs): Offline synthetic fixture store implementing `ReadOnlyStore`.
- [`src/store/sqlite.rs`](src/store/sqlite.rs): SQLite backend implementing `ReadOnlyStore`, compatible with local stores and interoperable with `wacli` database mirrors.
- [`src/watcher.rs`](src/watcher.rs): Streaming and filtering engine supporting JSON Lines output, group filters, keyword queries, and follow/poll loops.
- [`src/cli.rs`](src/cli.rs): Clap-derived CLI definition with subcommands `auth`, `sync`, `messages`, `chats`, `groups`, `watch`. **Zero send paths.**
- [`src/main.rs`](src/main.rs): CLI runner with JSON / plain-text formatting and exit code handling.
- [`tests/no_send_test.rs`](tests/no_send_test.rs): Tests verifying compile-time and runtime absence of any `send` command.
- [`tests/watcher_tests.rs`](tests/watcher_tests.rs): Tests validating JSON Lines formatting, group filtering, keyword queries, and limits.
- [`tests/cli_tests.rs`](tests/cli_tests.rs): End-to-end integration tests for all CLI subcommands against synthetic fixtures.
- [`tests/store_tests.rs`](tests/store_tests.rs): Tests validating `FixtureStore` and `SqliteStore` operations.

---

## 2. Hard Constraints Verification

1. **No `send` subcommand in public CLI**:
   - `clap` command definition contains no `send` subcommand (`tests/no_send_test.rs`).
   - `--help` displays only read-only commands (`auth`, `sync`, `messages`, `chats`, `groups`, `watch`).
   - Executing `wa-ro send` yields `error: unrecognized subcommand 'send'` (exit code 2).
2. **No secrets in repository**:
   - Authentication metadata and SQLite databases default to XDG data directories (`~/.local/share/wa-ro/` via `dirs::data_dir()`).
3. **No remote repository creation, push, merge, or deploy**:
   - Only local development performed within `/srv/Projects/wa-ro`.
4. **Offline operation**:
   - Default bundled fixtures allow `watch`, `search`, and `list` commands to run 100% offline without network or running daemon.

---

## 3. How to Run Tests & Verification

### Running Unit & Integration Tests (19 tests)
```bash
cargo test
```
Result: All 19 tests pass (0 failures, 0 ignored).

### Running Clippy
```bash
cargo clippy --all-targets -- -D warnings
```
Result: Clean execution, 0 warnings.

### Verifying Success Criteria Command
```bash
cargo run -- watch --group futsal --json
```
Output (6 JSON Lines):
```json
{"id":"MSG-FUT-001","chat_jid":"120363028471928374@g.us","chat_name":"Futsal Terça 21h - Carnaxide","sender_jid":"351911111111@s.whatsapp.net","sender_name":"Carlos Silva","timestamp":"2026-10-23T09:00:00Z","text":"Boas malta, jogo amanhã às 21h confirmado?","is_from_me":false}
{"id":"MSG-FUT-002","chat_jid":"120363028471928374@g.us","chat_name":"Futsal Terça 21h - Carnaxide","sender_jid":"351911222333@s.whatsapp.net","sender_name":"Pedro Santos","timestamp":"2026-10-23T09:10:00Z","text":"Sim, campo 2 reservado. Quem vai?","is_from_me":false,"quoted_message_id":"MSG-FUT-001"}
...
```

### Verifying Messages Search
```bash
cargo run -- messages search "reunião"
```
Output:
```
[2026-10-23 08:45:00 UTC] [Escola Básica 3º Ano - Turma B] Delegada Ana: Aviso: a reunião de pais foi remarcada para sexta-feira às 18h30.
```
