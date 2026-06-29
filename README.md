# KrustyRabz Poker

A multiplayer poker platform with a Rust game server, Rocket web client, and shared core library.

---

## Design Overview

The project is a **Cargo workspace** with four crates that separate concerns and share types via the `poker_core` library.

```
poker-project-krustyrabz/
├── core/      # Shared domain types, protocol, and hand evaluation
├── server/    # Game host (Rocket HTTP + WebSocket)
├── client/    # Web client (Rocket + Maud + HTMX + reqwest)
└── storage/   # Persistence (SeaORM, Supabase/PostgreSQL)
```

- **core** — Single source of truth for cards, hands, protocol messages, and game types. No I/O; used by both server and client.
- **server** — Runs games, holds state (House), and exposes HTTP and WebSocket endpoints. Depends on `core` and optionally `storage`.
- **client** — Web UI service with auth/session, lobby, create/join/watch/leave flows, and in-game actions. Depends on `core`; talks to server over HTTP.
- **storage** — Database connection and migrations (e.g. user accounts). Used by server/auth flows.

---

## Core (`poker_core`)

Shared library used by server and client so both sides use the same types and rules.

| Module       | Role                                                                                                                                                                                                                                                               |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **card**     | `Card`, `Rank`, `Suit`, `CardType` — deck representation.                                                                                                                                                                                                          |
| **hand**     | `Hand`, `HandRank`, `HandCategory` — 5- and 7-card evaluation (high card through royal flush).                                                                                                                                                                     |
| **betting**  | `BetAction`, `BettingOutcome` — bet/check/call/raise/fold.                                                                                                                                                                                                         |
| **player**   | `Player` — id, hand, chips, game_id; drawing from a `DeckTrait`.                                                                                                                                                                                                   |
| **table**    | `Table` — seats players (up to 5), add/remove by `Player` or id.                                                                                                                                                                                                   |
| **protocol** | Client–server **contract**: request/response types (JSON) for HTTP. Defines `CreateGameRequest`, `JoinGameRequest`, `ActionRequest`, `GameResponse`, `GameStateUpdate`, `GameType`, `GameStatus`, `BettingRound`, `GameAction`, etc. Serialization is via `serde`. |

Protocol lives in core so the server implements and the client consumes the same API shape without duplication.

---

## Server

- **Stack:** Rocket HTTP API + Server-Sent Events push updates, in-memory game state, storage-backed account/stat updates.
- **Entry:** `server/src/main.rs` builds Rocket, enables CORS for the web client (port 8001), initializes DB/Supabase clients, and starts a timeout checker.
- **House:** `server/src/house.rs` manages live games, event channels, and player timeout handling.
- **Game variants:** `server/src/game.rs` supports `FiveCardDraw`, `SevenCardStud`, and `TexasHoldEm`.
- **Routes:** Includes game lifecycle and account endpoints: list/create/join/leave, start hand, game state fetch, perform action, add chips, get stats, register viewer, house rules, and Server-Sent Events game events.
- **Storage integration:** On showdown, winner balances are persisted via the `storage` repository.

---

## Client

- **Stack:** Rocket + Maud + HTMX for server-rendered pages, with `reqwest` for API calls to the game server.
- **Entry:** `client/src/main.rs` launches a web app (default port 8001), mounts page/action routes, and serves static assets from `client/public`.
- **API layer:** `client/src/api/client.rs` contains `PokerClient`, mapping UI actions to server endpoints (`list_games`, `create_game`, `join_game`, `start_hand`, `perform_action`, `add_chips`, `register_viewer`, etc.).
- **Auth/session:** `client/src/authentication` and cookie helpers in `main.rs` handle login/register and signed session cookies.
- **Routes:** `client/src/routes` includes `login`, `register`, `create_game`, `join_game`, `game`, `watch_game`, `chips`, and `leave_game`.
- **Create flow updates:** Create table now supports selecting game variant (Five Card Draw / Seven Card Stud / Texas Hold'em) and redirects directly into `/play_game` on success.

Design choice: client is a web frontend gateway that delegates poker rules/state to the server and relies on `poker_core` for shared protocol types.

---

## Storage

- **Role:** Database access and migrations for user accounts and related data.
- **Stack:** SeaORM, PostgreSQL (e.g. Supabase); `dotenv` for `DATABASE_URL`.
- **Layout:** `entities/` (e.g. user account), `migration/`, `repository.rs` for queries. Optional use by server and auth flows.

---

## Crates Used

Workspace crates:

- **client** — CLI client.
- **poker_core** — Shared domain types and protocol.
- **server** — Game host and HTTP/WebSocket API.
- **storage** — Persistence and migrations.

External crates (across workspace):

- **arrayvec** — Fixed-capacity vectors in core.
- **async-trait** — Async traits in storage.
- **dashmap** — Concurrent map utility used by the client crate.
- **dotenv** — Load environment variables for DB/auth config.
- **futures-util** — Async utilities for WebSocket handling.
- **maud** — HTML template engine for the Rocket web client.
- **rand** — Randomness for decks/shuffling.
- **reqwest** — HTTP client used by the web client API wrapper.
- **rocket** — HTTP server framework.
- **rocket_ws** — WebSocket support for Rocket.
- **sea-orm** — ORM for storage.
- **sea-orm-migration** — Database migrations.
- **serde** — Serialization and deserialization.
- **serde_json** — JSON handling.
- **sqlx** — SQL access (Postgres).
- **strum** — Enum utilities.
- **strum_macros** — Derive macros for enums.
- **supabase_rs** — Supabase client.
- **tokio** — Async runtime.
- **uuid** — Unique IDs with serde support.

---

## Data Flow (High Level)

1. **Browser** loads the Rocket client app on port 8001.
2. **Client service** sends HTTP requests (create/join/start/action/watch/chips) using `PokerClient`, with payloads from `poker_core::protocol`.
3. **Server** updates `House` + selected `Game` variant, returns protocol responses, and pushes updates over Server-Sent Events where applicable.
4. **core** provides shared types/hand logic so client and server remain in sync.

---

## Running

- **Server:** From repo root, `cargo run -p server` (ensure port 8000 or configured host is free).
- **Client:** `cargo run -p client` (web client serves on `http://127.0.0.1:8001` by default).
- **Tests:** `cargo test -p poker_core` for core unit tests (cards, hands, protocol).

Requires a Rust toolchain and, for storage, a `.env` with `DATABASE_URL` (and any auth keys if using Supabase).
