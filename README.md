# KrustyRabz Poker

Multiplayer poker, built in Rust. The server runs the table, the browser client renders the game, and a shared crate keeps the rules and API types from drifting apart.

It supports Five Card Draw, Seven Card Stud, and Texas Hold'em—because shipping one poker variant would have been suspiciously responsible.

## What you get

- Create, join, leave, and spectate live games
- Play Five Card Draw, Seven Card Stud, or Texas Hold'em
- Real-time game updates with Server-Sent Events
- Login, registration, sessions, chip balances, and player stats
- Shared card, hand-evaluation, betting, and protocol types
- PostgreSQL/Supabase-backed account persistence

## Stack

| Layer | Built with |
| --- | --- |
| Web client | Rocket, Maud, HTMX, `reqwest` |
| Game server | Rocket, Tokio, Server-Sent Events |
| Game domain | Rust, `serde`, `rand` |
| Persistence | SeaORM, PostgreSQL, Supabase |

## Project layout

```text
.
├── client/   # Browser-facing Rocket app and game UI (port 8001)
├── server/   # Poker API, live game state, and event stream (port 8000)
├── core/     # Cards, hands, betting, tables, and shared API contract
└── storage/  # SeaORM models, migrations, and Supabase integration
```

The important part: `core` owns the rules and request/response types. The client and server both depend on it, so they cannot quietly invent incompatible versions of poker.

## Run it locally

### 1. Prerequisites

- A current Rust toolchain (`rustup` is the easy route)
- A PostgreSQL database—Supabase works
- A Supabase project for authentication

### 2. Configure environment variables

Create a `.env` file at the repository root:

```dotenv
DATABASE_URL=postgresql://USER:PASSWORD@HOST:5432/DATABASE
SUPABASE_URL=https://YOUR_PROJECT.supabase.co
SUPABASE_KEY=YOUR_SUPABASE_KEY

# Optional. This is already the default for local development.
SERVER_URL=http://127.0.0.1:8000
```

The server initializes the database and Supabase clients at startup, so all three required variables must be present before you launch it.

### 3. Apply migrations

```bash
cargo run -p storage --bin migrate -- up
```

### 4. Start the server

```bash
cargo run -p server
```

The game API listens on `http://127.0.0.1:8000`.

### 5. Start the client

In a second terminal:

```bash
cargo run -p client
```

Open [http://127.0.0.1:8001](http://127.0.0.1:8001), make an account, create a table, and start bluffing.

## Development commands

```bash
# Run the full workspace test suite
cargo test --workspace

# Focus on game rules and shared protocol types
cargo test -p poker_core

# Check the workspace without running it
cargo check --workspace

# Format the codebase
cargo fmt --all

# Run lints for every workspace target
cargo clippy --workspace --all-targets
```

### Pre-commit checks

The repository includes pre-commit checks for formatting and Clippy. Install
[`pre-commit`](https://pre-commit.com/), then enable the hook once per clone:

```bash
pre-commit install
```

## How a move reaches the table

```text
Browser UI
    ↓ HTTP request (shared protocol types)
Client app
    ↓
Poker server → House → active game
    ↓                    ↓
SSE update ← game state / betting / showdown
    ↓
Browser UI refreshes
```

The server owns the live table state and enforces game actions. The client is intentionally thin: it asks for things, renders the answer, and lets the server be the house.

## API at a glance

The server exposes JSON endpoints for the core game flow:

| Action | Endpoint |
| --- | --- |
| List or create tables | `GET /games`, `POST /games` |
| Join or leave | `POST /games/:game_id/join`, `POST /games/:game_id/leave` |
| Start a hand or act | `POST /games/:game_id/start`, `POST /games/:game_id/action` |
| Read game state | `GET /games/:game_id?player_id=...` |
| Subscribe to updates | `GET /games/:game_id/events` |

The complete request and response contract lives in [`core/src/protocol.rs`](core/src/protocol.rs).

## Contributing

Keep rules and API changes in `core` when they are shared across the client and server. Run formatting and tests before opening a PR. Small, focused changes beat a 1,400-line “quick cleanup.”

## License

No license has been specified yet. Do not assume reuse is permitted until one is added.
