# KrustyRabz Poker

A multiplayer poker platform with a Rust server, CLI client, and shared core library.

---

## Design Overview

The project is a **Cargo workspace** with four crates that separate concerns and share types via the `poker_core` library.

```
poker-project-krustyrabz/
├── core/      # Shared domain types, protocol, and hand evaluation
├── server/    # Game host (Rocket HTTP + WebSocket)
├── client/    # CLI client (reqwest + auth)
└── storage/   # Persistence (SeaORM, Supabase/PostgreSQL)
```

- **core** — Single source of truth for cards, hands, protocol messages, and game types. No I/O; used by both server and client.
- **server** — Runs games, holds state (House), and exposes HTTP and WebSocket endpoints. Depends on `core` and optionally `storage`.
- **client** — Terminal UI: auth (register/login), list/create/join games, play, and watch. Depends on `core`; talks to server over HTTP.
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

- **Stack:** Rocket (HTTP), optional WebSockets (`rocket_ws`), internal game state.
- **Entry:** `main.rs` builds Rocket, mounts routes, and manages a `House` state.
- **House:** Holds active games (e.g. pending, 2–5 player buckets). Creates games by variant (Five Card Draw, Seven Card Stud, Texas Hold'em), matches join/create/action/viewer requests to the right game.
- **Game variants:** `game.rs` defines an enum (`Game::FiveCardDraw`, `SevenCardStud`, `TexasHoldEm`) and per-variant logic: deck, table, pot, betting rounds, deal, and actions.
- **Routes:** Typically list games, create game, join game, get game state, perform action, get stats, register viewer, get house rules — all using types from `poker_core::protocol`.
- **Storage:** Can use the `storage` crate for user/account persistence (e.g. Supabase/PostgreSQL via SeaORM).

---

## Client

- **Stack:** `reqwest` (async HTTP), terminal I/O.
- **Entry:** `main.rs` — banner, auth loop (register/login/exit), then main menu: list & join games, create game, watch game, logout, exit.
- **API:** `api/client.rs` — `PokerClient` with methods that map to server routes (e.g. `list_games`, `create_game`, `join_game`, `perform_action`). Uses protocol types from `poker_core`.
- **Auth:** `authentication/` — register, login, session (e.g. Supabase Auth); session used for authenticated requests.
- **Games:** `games/game_settings` — create or join a game then run the play loop (prompts, send actions, show state).
- **Viewer:** `viewer/` — watch a game (e.g. by game id) without playing.
- **Player:** `player/` — local models for account and statistics (optional; server may be source of truth).

Design choice: client is a thin CLI that delegates rules and state to the server and uses `poker_core` only for types and protocol.

---

## Storage

- **Role:** Database access and migrations for user accounts and related data.
- **Stack:** SeaORM, PostgreSQL (e.g. Supabase); `dotenv` for `DATABASE_URL`.
- **Layout:** `entities/` (e.g. user account), `migration/`, `repository.rs` for queries. Optional use by server and auth flows.

---

## Data Flow (High Level)

1. **Client** sends HTTP requests (create game, join game, action) with bodies that match `poker_core::protocol` request types.
2. **Server** parses them, updates `House` and the right `Game` variant, and returns responses using protocol response types (`GameResponse`, `GameStateUpdate`, etc.).
3. **core** is not involved at runtime in I/O; it only defines the types and hand-evaluation logic so server and client stay in sync.

---

## Running

- **Server:** From repo root, `cargo run -p server` (ensure port 8000 or configured host is free).
- **Client:** `cargo run -p client` (defaults to `http://127.0.0.1:8000`).
- **Tests:** `cargo test -p poker_core` for core unit tests (cards, hands, protocol).

Requires a Rust toolchain and, for storage, a `.env` with `DATABASE_URL` (and any auth keys if using Supabase).
