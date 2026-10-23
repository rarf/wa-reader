# wa-ro — WhatsApp Read-Only CLI

> **Strictly read-only WhatsApp CLI for humans, Grok Bot, and Chief of Staff (CoS) agents.**
> Inspired by and designed as a read-only Rust port/companion to [`wacli`](https://github.com/openclaw/wacli) (Go + whatsmeow).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.99%2B-orange.svg)]()
[![Safety](https://img.shields.io/badge/safety-no--send--guaranteed-success.svg)]()

---

## 1. Goal & Motivation

When integrating Large Language Models (LLMs), AI assistants, or autonomous bots (such as **Grok Bot / Chief of Staff**) with messaging systems, **fear of the bot accidentally sending an unintended WhatsApp message to a boss, client, school, or sports group is existential**.

`wa-ro` solves this by introducing a **compile-time invariant: zero outbound messaging paths**.
- There is **no `send` subcommand** in the CLI.
- There is **no send function, method, or outbound network handler** in the crate.
- Read operations (chats, groups, historical message search, real-time watchers) are completely uninhibited.

---

## 2. Secção de Segurança (pt-PT) / Safety Guarantee

> ### 🇵🇹 Garantia de Segurança Inviolável (Read-Only por Construção)
> O `wa-ro` foi desenhado especificamente para permitir que agentes de IA e pipelines de automação tenham visibilidade total das conversas e grupos de WhatsApp **sem qualquer risco de envio acidental**.
>
> 1. **Ausência em tempo de compilação**: O comando `send` não existe nem está oculto atrás de flags ou configurações. É fisicamente impossível compilar um comando de envio através deste binário.
> 2. **Sem segredos no repositório**: Credenciais de sessão e ficheiros de base de dados são armazenados exclusivamente nas diretorias standard do utilizador (XDG Data Dirs: `~/.local/share/wa-ro/`).
> 3. **Isolamento de agentes**: Um agente LLM que invoque o `wa-ro` só consegue listar, pesquisar e escutar eventos (JSON Lines). Nunca poderá responder ou interagir externamente.

---

## 3. Use Cases

### Use Case 1: Watch a Futsal / Football Group → Alerts
- Monitor casual sports groups (e.g., "Futsal Terça 21h - Carnaxide").
- Alert Chief of Staff when games are scheduled, quorums are incomplete (*"Faltam 2 para fechar os 10"*), or last-minute dropouts occur.
```bash
wa-ro watch --group "futsal" --json
```

### Use Case 2: Watch a School / Children Group → Alerts for Chief of Staff
- Monitor school and class parents groups (e.g., "Escola Básica 3º Ano - Turma B").
- Filter for urgent notifications: schedule updates (*"Aviso: a reunião de pais foi remarcada"*), homework deadlines (*"Trabalho de Estudo do Meio para entregar dia 28"*), or school transport changes (*"carrinha sai às 08h15"*).
```bash
wa-ro watch --group "Escola Básica" --json
```

### Future Use Case: Family / Kids Caretaker Bot Template
- Monitor family chats to automatically summarize pickup times, sports drop-offs, and afternoon activities.

---

## 4. Architecture & whatsmeow Research

### Rust Ecosystem Analysis: WhatsApp Web Multi-Device
In the Go ecosystem, [`tulir/whatsmeow`](https://github.com/tulir/whatsmeow) is the industry-standard library powering projects like `openclaw/wacli`. For Rust, research yields three approaches:

1. **`whatsapp-rust` (`oxidezap/whatsapp-rust` / crates.io `whatsapp-rust`)**:
   - A pure-Rust asynchronous implementation inspired by `whatsmeow` and `Baileys`.
   - Supports QR pairing, Signal Protocol E2EE handshakes, WebSocket transport, and modular SQLite storage.
2. **`whatsmeow` / `whatsmeow-sys` (Crates.io)**:
   - Idiomatic Rust bindings calling the upstream Go `whatsmeow` engine via an FFI bridge.
   - Advantage: 100% upstream protocol parity with Go whatsmeow.
3. **`Whatsmeow-rust` (`YASSERRMD/Whatsmeow-rust`)**:
   - Clean-room Rust port implementing binary XML, Noise Protocol XX handshakes, and Curve25519 cryptography.

### `wa-ro` Architectural Design
To prioritize a **solid scaffold and reproducible offline operation** over an unstable live reverse-engineered protocol stack, `wa-ro` adopts a clean trait-based architecture:

```
                  ┌───────────────────────────────┐
                  │          wa-ro CLI            │
                  │ (clap derive, no send verbs)  │
                  └──────────────┬────────────────┘
                                 │
                 ┌───────────────▼───────────────┐
                 │    trait ReadOnlyStore        │
                 │ (auth, list, search, watch)   │
                 └───────┬───────────────┬───────┘
                         │               │
        ┌────────────────▼──────┐ ┌──────▼─────────────────────┐
        │     FixtureStore      │ │        SqliteStore         │
        │ (bundled/offline JSON │ │ (XDG ~/.local/share/wa-ro/ │
        │   synthetic data)     │ │  or bridge ~/.wacli.db)    │
        └───────────────────────┘ └────────────────────────────┘
```

#### Phased Roadmap
- **Phase 1 (Milestone A — Current)**:
  - Rust CLI with subcommands: `auth`, `sync`, `messages`, `chats`, `groups`, `watch`.
  - Core domain models (`Chat`, `Group`, `Message`, `AuthStatus`, `WatchFilter`).
  - Synthetic offline fixtures (`sample_store.json`) covering all target use cases.
  - Native SQLite storage backend + automatic bridge discovery of `wacli` database (`~/.wacli/wacli.db`).
  - Comprehensive unit and integration test suite proving the no-send invariant and JSON Lines watcher.
- **Phase 2 (Milestone B — wacli Bridge)**:
  - Real-time file-watcher / WAL sync reader on `~/.wacli/wacli.db` to stream updates from an existing `wacli sync --follow` session.
- **Phase 3 (Milestone C — Native Companion Session)**:
  - Integration of `whatsapp-rust` or Go FFI companion bridge strictly in listener mode (outbound send calls stripped or dead-coded).

---

## 5. Command Reference

### Companion Pairing & Status (`auth`)
```bash
# Display companion authentication status
wa-ro auth status
wa-ro auth status --json

# Initiate companion pairing flow (stub)
wa-ro auth pair
wa-ro auth pair --phone "+351912345678"
```

### Synchronization (`sync`)
```bash
# Sync local mirror from companion session
wa-ro sync
wa-ro sync --once
```

### Inspecting & Searching Messages (`messages`)
```bash
# List recent messages (optionally filter by chat)
wa-ro messages list
wa-ro messages list --chat "futsal" --limit 10

# Full-text search across messages, senders, and chat names
wa-ro messages search "reunião"
wa-ro messages search "coletes" --json

# Inspect single message details
wa-ro messages show MSG-ESC-001
wa-ro messages show MSG-ESC-001 --json
```

### Listing Chats & Groups (`chats` / `groups`)
```bash
# List conversation threads
wa-ro chats list
wa-ro chats list --query "Escola" --json

# List groups and topics
wa-ro groups list
wa-ro groups list --json
```

### JSON Lines Watcher (`watch`)
Designed specifically for piping into Unix tools (`jq`, `awk`) or consumption by autonomous AI agents (Chief of Staff):
```bash
# Watch futsal group messages in JSON Lines format
wa-ro watch --group "futsal" --json

# Watch school group for specific keywords
wa-ro watch --group "Escola Básica" --query "reunião" --json

# Human-readable watcher format
wa-ro watch --group "futsal"

# Continuous streaming mode
wa-ro watch --group "futsal" --follow --interval-ms 1000
```

---

## 6. Development & Verification

### Running Tests
```bash
cargo test
```

### Running Clippy (Zero Warnings Enforcement)
```bash
cargo clippy --all-targets -- -D warnings
```

### Verifying No-Send Invariant
```bash
# Running 'send' produces an unrecognized subcommand error
cargo run -- send "test"
# Exit code: 2
```

---

## 7. License

MIT License. See [LICENSE](LICENSE) for details.
