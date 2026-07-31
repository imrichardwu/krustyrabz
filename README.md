# 🃏 KrustyRabz

**Multiplayer poker, written entirely in Rust.** The server runs the table, the browser client renders the game, and a shared crate keeps the rules and API types from drifting apart.

Play **Five Card Draw**, **Seven Card Stud**, or **Texas Hold'em** — create a table, join, leave, or spectate — with every connected browser staying in sync in real time.

---

## Why it's built this way

The server is the **house**. It owns the deck, enforces every rule, and is the single source of truth for table state. The client is intentionally dumb: it submits actions and renders whatever the server sends back. No game logic lives in the browser, so there's nothing to tamper with.

The most important design decision is the **shared `core` crate**. Card logic, hand evaluation, betting rules, and the request/response types all live in one place that both the server and client depend on. If the API changes, both sides fail to compile until they agree again — the type system makes protocol drift impossible.

---

## Features

- ♠️ **Three poker variants** — Five Card Draw, Seven Card Stud, Texas Hold'em
- 👥 **Live multiplayer** — create, join, leave, and spectate live games
- ⚡ **Real-time updates** — table state pushed to every client via Server-Sent Events (SSE)
- 🔒 **Server-authoritative** — all rules enforced server-side; the client can't cheat
- 💾 **Persistent accounts** — user accounts and game history stored in PostgreSQL

---

## Architecture

```
┌──────────────┐        actions         ┌──────────────┐
│  Web Client  │ ─────────────────────▶ │  Game Server │
│  Rocket +    │                        │  Rocket +    │
│  Maud + HTMX │ ◀───── SSE stream ──── │  Tokio       │
└──────────────┘                        └──────┬───────┘
                                               │
                        ┌──────────────────────┴───────────┐
                        │        core (shared crate)        │
                        │  card rules · hand eval · betting │
                        │       · API request/response      │
                        └──────────────────────┬───────────┘
                                               │
                                        ┌──────┴───────┐
                                        │  SeaORM +    │
                                        │  PostgreSQL  │
                                        └──────────────┘
```

- **Web Client** — Rocket + [Maud](https://maud.lambda.xyz/) templating + [HTMX](https://htmx.org/) for interactivity
- **Game Server** — Rocket + Tokio handling live table state and event streaming
- **Game Core** — shared Rust library: card rules, hand evaluation, betting logic, and API contracts
- **Data Layer** — SeaORM with PostgreSQL / Supabase for accounts and persistence

---

## Tech Stack

| Layer      | Tech                                    |
| ---------- | --------------------------------------- |
| Language   | Rust                                    |
| Web / API  | Rocket, Tokio                           |
| Frontend   | Maud (templating), HTMX                 |
| Realtime   | Server-Sent Events (SSE)                |
| Database   | PostgreSQL / Supabase via SeaORM        |

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- PostgreSQL, or a [Supabase](https://supabase.com/) project
- Supabase credentials (if using Supabase)

### Setup

```bash
# 1. Clone the repo
git clone https://github.com/imrichardwu/krustyrabz.git
cd krustyrabz

# 2. Configure environment variables
cp .env.example .env
#    then fill in your database URL / Supabase credentials

# 3. Run database migrations
cargo run --bin migrate   # (adjust to your migration command)

# 4. Start the game server (port 8000)
cargo run --bin server

# 5. In a second terminal, start the web client (port 8001)
cargo run --bin client
```

Open **http://localhost:8001** and deal yourself in.

---

## License

No license has been specified yet. Do not assume reuse is permitted until one is added.
