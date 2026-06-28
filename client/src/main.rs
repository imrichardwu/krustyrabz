mod api;
mod authentication;
pub mod routes;

use crate::api::PokerClient;
pub use authentication::AuthSession;
use rocket::http::{Cookie, CookieJar, Header, SameSite};
use rocket::response::{Redirect, Responder};
use rocket::State;
use rocket::fs::FileServer;
use poker_core::{BettingRound, PlayerInfo};
use poker_core::GameStateUpdate;
use maud::{html, Markup, DOCTYPE, PreEscaped};

#[macro_use] extern crate rocket;

// ============================================================================
// Session helpers
// ============================================================================

pub fn get_session(
    cookies: &CookieJar<'_>,
) -> Option<AuthSession> {
    // Get session directly from cookie (serialized as JSON)
    let session_json = cookies.get_private("auth_session")?.value().to_owned();
    serde_json::from_str(&session_json).ok()
}

pub fn set_session_cookie(cookies: &CookieJar<'_>, auth_session: AuthSession) {
    // Serialize session to JSON and store in cookie
    if let Ok(session_json) = serde_json::to_string(&auth_session) {
        let mut c = Cookie::new("auth_session", session_json);
        c.set_http_only(true);
        c.set_same_site(SameSite::Lax);
        // Session expires in 3 hours
        c.set_max_age(rocket::time::Duration::hours(3));
        cookies.add_private(c);
    }
}

// ============================================================================
// Rocket launch
// ============================================================================

#[launch]
fn rocket() -> _ {
    let client = PokerClient::localhost();

    rocket::build()
        .mount("/", routes![
            landing,
            main_menu,
            logout,
            play_game,
            play_game_fragment,
        ])
        .mount("/", routes::login::routes())
        .mount("/", routes::register::routes())
        .mount("/", routes::game::routes())
        .mount("/", routes::create_game::routes())
        .mount("/", routes::join_game::routes())
        .mount("/", routes::watch_game::routes())
        .mount("/", routes::chips::routes())
        .mount("/", routes::leave_game::routes())
        .mount("/public", FileServer::from("public"))
        .manage(client)
}

// ============================================================================
// HTMX redirect helper
// ============================================================================

#[derive(Responder)]
pub struct HxRedirect {
    inner: String,
    header: Header<'static>,
}

impl HxRedirect {
    pub fn to(url: impl Into<String>) -> Self {
        HxRedirect {
            inner: String::new(),
            header: Header::new("HX-Redirect", url.into()),
        }
    }
}

// ============================================================================
// Card rendering helpers
// ============================================================================

/// Render a face-up playing card from a code like "AS", "KH", "10D", "QC".
/// Suit is always the last character (ASCII letter); rank is everything before it.
pub fn render_card(card_str: &str) -> Markup {
    // Use char-boundary-safe split: subtract the byte length of the suit char
    let suit = card_str.chars().last().unwrap_or('?');
    let rank = &card_str[..card_str.len() - suit.len_utf8()];
    let (symbol, is_red) = match suit {
        'S' => ("♠️", false),
        'H' => ("♥️", true),
        'D' => ("♦️", true),
        'C' => ("♣️", false),
        _   => ("?", false),
    };
    let color = if is_red { "text-red-600" } else { "text-slate-800" };
    html! {
        div class="inline-flex flex-col justify-between bg-white rounded-lg border border-slate-200 shadow p-1.5 m-1 select-none cursor-default"
            style="width:3rem; height:4.2rem; min-width:3rem;" {
            span class=(format!("text-xs font-bold leading-none {}", color)) { (rank) }
            span class=(format!("text-xl leading-none text-center {}", color)) { (symbol) }
            span class=(format!("text-xs font-bold leading-none self-end rotate-180 {}", color)) { (rank) }
        }
    }
}

/// Render a face-down card back. `color` is "red" or "blue".
pub fn render_card_back(color: &str) -> Markup {
    let bg = if color == "red" { "#7c1d1d" } else { "#1e3a5f" };
    html! {
        div class="inline-flex rounded-lg shadow m-1 select-none"
            style=(format!("width:3rem; height:4.2rem; min-width:3rem; background:{};", bg)) {
            div class="m-1 flex-1 rounded flex items-center justify-center"
                style="border:1px solid rgba(255,255,255,0.15);" {
                span class="text-white/30 text-lg" { "🂠" }
            }
        }
    }
}

// ============================================================================
// Base layout
// ============================================================================

fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8" {}
                meta name="viewport" content="width=device-width, initial-scale=1" {}
                title { (title) }
                script src="https://cdn.tailwindcss.com" {}
                script src="https://unpkg.com/htmx.org@2.0.0" {}
                script {(PreEscaped(r#"
document.addEventListener('htmx:afterRequest', function(e) {
    var t = e.detail.target;
    if (t && t.tagName === 'DIALOG') t.showModal();
});
"#))}
            }
            body class="min-h-screen text-slate-200" style="font-family:Avenir,Helvetica,Arial,sans-serif; background:#0f1117;" {
                (content)
            }
        }
    }
}

// ============================================================================
// Landing page
// ============================================================================

#[get("/")]
async fn landing() -> Markup {
    layout("Poker - Welcome", html! {
        div class="min-h-screen flex flex-col items-center justify-center px-4 py-12 gap-10" {
            div class="text-center" {
                h1 class="text-5xl font-bold mb-3" style="color:#42b883;" {
                    "POKER HOUSE"
                }
                p class="text-slate-400 text-lg" { "Play. Bluff. Win big. Or go broke trying." }
            }

            div class="flex gap-2 justify-center" {
                (render_card_back("red"))
                (render_card_back("blue"))
                (render_card_back("red"))
                (render_card_back("blue"))
                (render_card_back("red"))
            }

            div class="flex gap-4" {
                a href="/register_page"
                    class="px-8 py-3 rounded-lg font-semibold text-base transition-colors"
                    style="border:2px solid #42b883; color:#42b883; background:transparent;"
                    onmouseover="this.style.background='rgba(66,184,131,0.12)'"
                    onmouseout="this.style.background='transparent'" {
                    "REGISTER"
                }
                a href="/login_page"
                    class="px-8 py-3 rounded-lg font-semibold text-base transition-colors"
                    style="background:#42b883; color:#0f1117;"
                    onmouseover="this.style.background='#33a070'"
                    onmouseout="this.style.background='#42b883'" {
                    "LOGIN"
                }
            }

            p class="text-slate-600 text-xs text-center max-w-xs" {
                "80% of gamblers quit just before they make it big. Stay in the game."
            }
        }
    })
}

// Auth forms and POST handlers are in routes::login and routes::register

#[get("/logout")]
async fn logout(
    cookies: &CookieJar<'_>,
) -> Redirect {
    // Remove both old and new session cookies
    cookies.remove_private(Cookie::from("session_id"));
    cookies.remove_private(Cookie::from("auth_session"));
    Redirect::to("/")
}

// ============================================================================
// Main menu
// ============================================================================

#[get("/main_menu")]
async fn main_menu(
    cookies: &CookieJar<'_>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let username = session.username.clone();

    Ok(layout("The krusty Rabz - Lobby", html! {
        div class="flex min-h-screen" {

            // ── Sidebar ──────────────────────────────────────────────────────
            aside class="fixed top-0 left-0 h-full w-60 flex flex-col z-10"
                style="background:#111827; border-right:1px solid #1e2d3d;" {

                // Logo
                a href="/main_menu"
                    class="px-6 py-5 flex items-center gap-2.5 transition-colors"
                    style="border-bottom:1px solid #1e2d3d;"
                    onmouseover="this.style.background='#1e2d3d'"
                    onmouseout="this.style.background='transparent'" {
                    span class="text-xl" style="color:#42b883;" { "P" }
                    div {
                        div class="font-bold text-white text-sm leading-tight" {
                            "The krusty Rabz"
                        }
                        div class="text-xs" style="color:#42b883;" { "Poker" }
                    }
                }

                // Nav
                nav class="flex-1 px-3 py-4 flex flex-col gap-0.5" {
                    a hx-get="/list_and_join_games" hx-target="#main-content" hx-swap="innerHTML"
                        class="flex items-center gap-3 px-3 py-2 rounded text-sm hover:bg-[#1e2d3d] text-gray-400 cursor-pointer" {
                        span { "GB" } span { "Game Browser" }
                    }
                    a hx-get="/list_and_join_games" hx-target="#main-content" hx-swap="innerHTML"
                        class="flex items-center gap-3 px-3 py-2 rounded text-sm hover:bg-[#1e2d3d] text-gray-400 cursor-pointer" {
                        span { "JR" } span { "Join Room" }
                    }
                    a hx-get="/create_new_game" hx-target="#main-content" hx-swap="innerHTML"
                        class="flex items-center gap-3 px-3 py-2 rounded text-sm hover:bg-[#1e2d3d] text-gray-400 cursor-pointer" {
                        span { "+" } span { "Create Room" }
                    }
                    a hx-get="/watch_game" hx-target="#main-content" hx-swap="innerHTML"
                        class="flex items-center gap-3 px-3 py-2 rounded text-sm hover:bg-[#1e2d3d] text-gray-400 cursor-pointer" {
                        span { "SP" } span { "Spectate" }
                    }
                    a hx-get="/add_chips" hx-target="#main-content" hx-swap="innerHTML"
                        class="flex items-center gap-3 px-3 py-2 rounded text-sm hover:bg-[#1e2d3d] text-gray-400 cursor-pointer" {
                        span { "$" } span { "Credit Bureau" }
                    }
                }

                // User + status
                div class="px-4 py-4 flex flex-col gap-3" style="border-top:1px solid #1e2d3d;" {
                    div class="flex items-center justify-between" {
                        span class="text-sm font-medium text-white" { (username) }
                        a href="/logout"
                            class="text-xs transition-colors"
                            style="color:#4a5568;"
                            onmouseover="this.style.color='#f87171'"
                            onmouseout="this.style.color='#4a5568'" {
                            "Sign out"
                        }
                    }
                    div class="flex items-center" {
                        span class="text-xs" style="color:#4a5568;" { "The krusty Rabz Poker" }
                    }
                }
            }

            // ── Main content area ─────────────────────────────────────────────
            main class="flex-1 ml-60 min-h-screen" style="background:#0f1117;" {
                div id="main-content" class="min-h-screen flex flex-col items-center justify-center p-12" {
                    // Hero
                    div class="text-center mb-10" {
                        h1 class="text-5xl font-bold mb-1" style="color:white;" {
                            "The krusty Rabz"
                        }
                        p class="text-lg font-medium mb-1" style="color:#42b883;" { "Poker" }
                        p class="text-sm" style="color:#4a5568;" {
                            "Welcome to the ultimate poker experience!"
                        }
                    }

                    // Game info card
                    div class="rounded-2xl p-8 mb-8 w-full max-w-md"
                        style="background:#111827; border:1px solid #1e2d3d;" {
                        div class="flex justify-between items-start mb-6" {
                            div {
                                div class="text-xs uppercase tracking-widest mb-1" style="color:#4a5568;" { "Game Type" }
                                div class="text-2xl font-bold text-white" {
                                    "Five Card Draw"
                                }
                                div class="text-xs mt-0.5" style="color:#4a5568;" { "Classic poker variant" }
                            }
                            div class="text-right" {
                                div class="text-xs uppercase tracking-widest mb-1" style="color:#4a5568;" { "Players" }
                                div class="text-2xl font-bold" style="color:#42b883;" { "2–10" }
                                div class="text-xs mt-0.5" style="color:#4a5568;" { "Players per table" }
                            }
                        }
                        // Play now = open create room
                        button
                            hx-get="/create_new_game"
                            hx-target="#main-content"
                            hx-swap="innerHTML"
                            class="w-full py-3 rounded-xl font-bold text-base mb-3 cursor-pointer transition-colors"
                            style="background:white; color:#0f1117;"
                            onmouseover="this.style.background='#e5e7eb'"
                            onmouseout="this.style.background='white'" {
                            "Play Now"
                        }
                        button
                            hx-get="/list_and_join_games"
                            hx-target="#main-content"
                            hx-swap="innerHTML"
                            class="w-full py-3 rounded-xl font-semibold text-sm cursor-pointer transition-colors"
                            style="background:transparent; color:#42b883; border:1px solid #42b883;"
                            onmouseover="this.style.background='rgba(66,184,131,0.08)'"
                            onmouseout="this.style.background='transparent'" {
                            "Browse Games"
                        }
                    }

                    // Card preview
                    div class="flex gap-1 justify-center" {
                        (render_card("AS")) (render_card("KH")) (render_card("QD"))
                        (render_card("JC")) (render_card("10H"))
                    }
                }
            }
        }
    }))
}

// ============================================================================
// Game play view helpers
// ============================================================================

fn render_opponent_panel(player: &PlayerInfo, action_on: Option<&str>) -> Markup {
    let is_acting = action_on == Some(player.username.as_str());
    let dimmed = if player.folded { "opacity-40" } else { "" };
    html! {
        div class=(format!("flex flex-col items-center rounded-xl p-4 gap-2 {}", dimmed))
            style=(if is_acting {
                "background:#1a2332; border:2px solid #f6c90e;"
            } else {
                "background:#1a2332; border:1px solid #2d3a4a;"
            }) {
            
            div class="flex items-center gap-2" {
                span class="text-sm font-semibold" style="color:white;" { (player.username) }
                @if player.is_dealer {
                    span class="text-xs px-1.5 py-0.5 rounded font-bold" style="background:#f6c90e; color:#0f1117;" { "D" }
                }
                @if is_acting {
                    span class="text-xs px-1.5 py-0.5 rounded font-bold animate-pulse" 
                        style="background:#3b82f6; color:white;" { "*" }
                }
            }

            div class="flex gap-1" {
                @for card_str in &player.face_up_cards {
                    (render_card(card_str))
                }
                @for _ in 0..(player.cards_count.saturating_sub(player.face_up_cards.len())) {
                    (render_card_back("blue"))
                }
            }

            div class="flex gap-3 text-xs" {
                span class="font-mono" style="color:#42b883;" { "$" (player.chips) }
                @if player.current_bet > 0 {
                    span class="font-mono" style="color:#f6c90e;" { "bet: $" (player.current_bet) }
                }
            }

            @if player.folded {
                span class="text-xs font-bold uppercase tracking-wide" style="color:#f87171;" { "Folded" }
            }
        }
    }
}

fn render_game_fragment(game_id: &str, session: &AuthSession, state: &GameStateUpdate) -> Markup {
    // action_on contains the username of the player whose turn it is
    let my_turn = state.action_on.as_deref() == Some(session.username.as_str());
    let is_drawing = state.betting_round == BettingRound::Drawing;
    let hand_started = !state.your_hand.is_empty();

    html! {
        div id="game-state"
            data-game-id=(game_id)
            data-player-id=(session.user_id)
            class="min-h-screen flex flex-col" style="background:#0f1117;" {

            // Top nav
            div class="flex items-center justify-between px-6 py-4 rounded-t-xl"
                style="background:#1a2332; border-bottom:1px solid #2d3a4a;" {
                div class="flex gap-4 items-center" {
                    a href="/main_menu" 
                        class="text-sm font-semibold transition-colors"
                        style="color:#42b883;"
                        onmouseover="this.style.color='#33a070'"
                        onmouseout="this.style.color='#42b883'" { "Lobby" }
                    form hx-post=(format!("/game/leave?game_id={}", game_id)) hx-confirm="Are you sure you want to leave this game?" {
                        button type="submit" 
                            class="text-sm font-semibold transition-colors"
                            style="color:#f87171;"
                            onmouseover="this.style.color='#ef4444'"
                            onmouseout="this.style.color='#f87171'" {
                            "Leave Game"
                        }
                    }
                }
                div class="flex gap-6 text-sm" {
                    span style="color:#7a8fa6;" { "Table: " span class="font-mono text-xs" style="color:white;" { (game_id.chars().take(8).collect::<String>()) "..." } }
                    span style="color:#7a8fa6;" { "Type: " span class="font-medium" style="color:white;" { (state.game_type) } }
                    span style="color:#7a8fa6;" { "Round: " span class="font-medium" style="color:white;" { (state.betting_round) } }
                    span class="font-semibold" style="color:#f6c90e;" { "Pot: $" (state.pot) }
                    span class="font-semibold" style="color:#42b883;" { "You: $" (state.your_chips) }
                }
                span class="text-xs" style="color:#4a5568;" { (session.username) }
            }

            // Winner message banner
            @if let Some(ref msg) = state.last_hand_message {
                div class="px-6 py-4 text-center font-semibold text-sm rounded-xl mb-4"
                    style="background:rgba(246,201,14,0.15); border:1px solid rgba(246,201,14,0.3); color:#f6c90e;" {
                    "Winner: " (msg)
                }
            }

            div class="flex-1 flex flex-col items-center justify-between p-6 gap-6" {

                // Opponents row
                div class="flex gap-3 justify-center flex-wrap w-full" {
                    @for player in state.players.iter().filter(|p| p.username != session.username) {
                        (render_opponent_panel(player, state.action_on.as_deref()))
                    }
                    @if state.players.iter().filter(|p| p.username != session.username).count() == 0 {
                        div class="text-sm italic" style="color:#4a5568;" { "Waiting for other players to join..." }
                    }
                }

                // Community cards (Texas Hold'em / Seven Card Stud)
                @if !state.community_cards.is_empty() {
                    div class="flex flex-col items-center gap-3" {
                        span class="text-xs uppercase tracking-widest" style="color:#4a5568;" { "Community Cards" }
                        div class="flex gap-2 rounded-xl p-4" style="background:#1a2332; border:1px solid #2d3a4a;" {
                            @for card in &state.community_cards {
                                (render_card(card))
                            }
                        }
                    }
                }

                // Player's hand
                div class="flex flex-col items-center gap-4" {
                    @if hand_started {
                        span class="text-xs uppercase tracking-widest" style="color:#4a5568;" { "Your Hand" }
                        div class="flex gap-2" {
                            @if is_drawing && my_turn {
                                @for (i, card) in state.your_hand.iter().enumerate() {
                                    label class="cursor-pointer group" {
                                        input type="checkbox" name="discard"
                                            id=(format!("discard-{}", i))
                                            value=(i)
                                            class="hidden peer" {}
                                        div class="peer-checked:opacity-40 peer-checked:ring-2 peer-checked:ring-red-500 peer-checked:rounded-lg transition-all" {
                                            (render_card(card))
                                        }
                                        span class="block text-center text-xs mt-1 peer-checked:font-semibold" 
                                            style="color:#4a5568;" { "discard" }
                                    }
                                }
                            } @else {
                                @for card in &state.your_hand {
                                    (render_card(card))
                                }
                            }
                        }
                    } @else {
                        div class="text-sm italic" style="color:#4a5568;" { "Hand not started yet" }
                    }
                }

                // Action panel
                div id="action-panel" class="flex gap-3 flex-wrap justify-center items-center" {

                    // Start hand button (shown when no hand in progress and enough players)
                    @if !hand_started && state.player_count >= 2 {
                        form hx-post="/game/start_hand" hx-target="#game-state" hx-swap="outerHTML" {
                            input type="hidden" name="game_id" value=(game_id) {}
                            button type="submit" 
                                class="px-6 py-3 rounded-lg font-bold text-sm transition-colors"
                                style="background:#42b883; color:#0f1117;"
                                onmouseover="this.style.background='#33a070'"
                                onmouseout="this.style.background='#42b883'" { 
                                "Start Hand" 
                            }
                        }
                    }

                    @if !hand_started && state.player_count < 2 {
                        span class="text-sm italic" style="color:#4a5568;" { "Need at least 2 players to start" }
                    }

                    // Betting actions (shown when it's my turn and hand is in progress, not drawing phase)
                    @if my_turn && hand_started && !is_drawing {
                        // Fold
                        form hx-post="/game/fold" hx-target="#game-state" hx-swap="outerHTML" {
                            input type="hidden" name="game_id" value=(game_id) {}
                            button type="submit" 
                                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-colors"
                                style="background:rgba(248,113,113,0.15); color:#f87171; border:1px solid rgba(248,113,113,0.3);"
                                onmouseover="this.style.background='rgba(248,113,113,0.3)'"
                                onmouseout="this.style.background='rgba(248,113,113,0.15)'" {
                                "Fold"
                            }
                        }

                        @if state.current_bet == 0 {
                            // Check (no bet to match)
                            form hx-post="/game/check" hx-target="#game-state" hx-swap="outerHTML" {
                                input type="hidden" name="game_id" value=(game_id) {}
                                button type="submit" 
                                    class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-colors"
                                    style="background:rgba(66,184,131,0.15); color:#42b883; border:1px solid rgba(66,184,131,0.3);"
                                    onmouseover="this.style.background='rgba(66,184,131,0.3)'"
                                    onmouseout="this.style.background='rgba(66,184,131,0.15)'" {
                                    "Check"
                                }
                            }

                            // Bet with amount (only when no one has bet yet)
                            form hx-post="/game/bet" hx-target="#game-state" hx-swap="outerHTML" class="flex gap-2 items-center" {
                                input type="hidden" name="game_id" value=(game_id) {}
                                input type="number" name="amount" min="1" placeholder="Amount"
                                    class="w-24 rounded-lg px-3 py-2.5 text-sm"
                                    style="background:#0f1117; border:1px solid #2d3a4a; color:white;" {}
                                button type="submit" 
                                    class="px-5 py-2.5 rounded-lg font-bold text-sm transition-colors"
                                    style="background:#f6c90e; color:#0f1117;"
                                    onmouseover="this.style.background='#d4a50a'"
                                    onmouseout="this.style.background='#f6c90e'" {
                                    "Bet"
                                }
                            }
                        } @else {
                            // Call
                            form hx-post="/game/call" hx-target="#game-state" hx-swap="outerHTML" {
                                input type="hidden" name="game_id" value=(game_id) {}
                                button type="submit" 
                                    class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-colors"
                                    style="background:rgba(59,130,246,0.15); color:#3b82f6; border:1px solid rgba(59,130,246,0.3);"
                                    onmouseover="this.style.background='rgba(59,130,246,0.3)'"
                                    onmouseout="this.style.background='rgba(59,130,246,0.15)'" {
                                    "Call $" (state.current_bet)
                                }
                            }

                            // Raise with amount (only when there's a bet to raise)
                            form hx-post="/game/raise" hx-target="#game-state" hx-swap="outerHTML" class="flex gap-2 items-center" {
                                input type="hidden" name="game_id" value=(game_id) {}
                                input type="number" name="amount" min="1" placeholder="Amount"
                                    class="w-24 rounded-lg px-3 py-2.5 text-sm"
                                    style="background:#0f1117; border:1px solid #2d3a4a; color:white;" {}
                                button type="submit" 
                                    class="px-5 py-2.5 rounded-lg font-bold text-sm transition-colors"
                                    style="background:rgba(249,115,22,0.2); color:#fb923c; border:1px solid rgba(249,115,22,0.4);"
                                    onmouseover="this.style.background='rgba(249,115,22,0.4)'"
                                    onmouseout="this.style.background='rgba(249,115,22,0.2)'" {
                                    "Raise"
                                }
                            }
                        }
                    }

                    // Draw action (Five Card Draw drawing phase)
                    @if my_turn && is_drawing {
                        form id="draw-form" hx-post="/game/draw" hx-target="#game-state" hx-swap="outerHTML" {
                            input type="hidden" name="game_id" value=(game_id) {}
                            div id="discard-inputs" {}
                            button type="submit"
                                class="px-6 py-3 rounded-lg font-bold text-sm transition-colors"
                                style="background:rgba(139,92,246,0.2); color:#a78bfa; border:1px solid rgba(139,92,246,0.4);"
                                onmouseover="this.style.background='rgba(139,92,246,0.4)'"
                                onmouseout="this.style.background='rgba(139,92,246,0.2)'" {
                                "Draw Selected Cards"
                            }
                        }
                        // JS copies checked discard checkboxes into the draw form on submit
                        script { (PreEscaped(r#"
document.getElementById('draw-form').addEventListener('htmx:configRequest', function(e) {
    var inputs = document.getElementById('discard-inputs');
    inputs.innerHTML = '';
    document.querySelectorAll('input[name="discard"]:checked').forEach(function(cb) {
        var h = document.createElement('input');
        h.type = 'hidden';
        h.name = 'discard_indices';
        h.value = cb.value;
        inputs.appendChild(h);
    });
});
"#)) }
                    }

                    // Waiting message when it's not my turn
                    @if !my_turn && hand_started && !is_drawing {
                        @if let Some(ref acting_username) = state.action_on {
                            span class="text-sm italic" style="color:#4a5568;" { "Waiting for " (acting_username) "..." }
                        }
                    }

                    // Timeout countdown when it's my turn
                    @if my_turn && hand_started {
                        div id="timeout-display" class="flex items-center gap-3 px-4 py-2 rounded-lg"
                            style="background:rgba(249,115,22,0.15); border:1px solid rgba(249,115,22,0.3);" {
                            span class="text-xl" style="color:#fb923c;" { "T" }
                            span id="countdown" class="font-mono text-sm font-semibold" style="color:#fb923c;" { "30s" }
                        }
                        script { (PreEscaped(r#"
(function() {
    let seconds = 30;
    const countdownEl = document.getElementById('countdown');
    const timeoutDisplayEl = document.getElementById('timeout-display');
    
    const interval = setInterval(function() {
        seconds--;
        if (countdownEl) {
            countdownEl.textContent = seconds + 's';
            
            // Change color as time runs out
            if (seconds <= 10) {
                timeoutDisplayEl.style.background = 'rgba(239,68,68,0.2)';
                timeoutDisplayEl.style.borderColor = 'rgba(239,68,68,0.4)';
                countdownEl.style.color = '#ef4444';
            }
            
            if (seconds <= 0) {
                clearInterval(interval);
                countdownEl.textContent = 'TIME OUT!';
            }
        } else {
            clearInterval(interval);
        }
    }, 1000);
})();
"#)) }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Play game (full page)
// ============================================================================

#[get("/play_game?<game_id>")]
async fn play_game(
    game_id: &str,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();

    let game_state = match client.get_game(game_id, &session_owned.user_id).await {
        Ok(s) => s,
        Err(_) => return Ok(layout("Error", html! {
            div class="p-8 text-center" {
                p class="text-red-400 mb-4" { "Could not load game state. Is the server running?" }
                a href="/main_menu" class="text-blue-400 hover:underline" { "Back to Lobby" }
            }
        })),
    };

    let gid = game_id.to_string();
    let fragment = render_game_fragment(game_id, &session_owned, &game_state);

    // SSE script: connect to server SSE, trigger HTMX fragment re-fetch on each event
    let sse_script = format!(r#"
(function() {{
    var gameId = {gid:?};
    var sseUrl = 'http://127.0.0.1:8000/games/' + gameId + '/events';
    console.log('Connecting to SSE:', sseUrl);
    
    var es = new EventSource(sseUrl);
    
    es.onopen = function() {{
        console.log('SSE connection established');
    }};
    
    es.onmessage = function(event) {{
        console.log('SSE event received:', event.data);
        htmx.ajax('GET', '/play_game_fragment?game_id=' + gameId, {{
            target: '#game-state',
            swap: 'outerHTML'
        }});
    }};
    
    es.onerror = function(err) {{
        console.error('SSE error:', err);
        console.warn('SSE disconnected, will retry automatically');
    }};
    
    window.addEventListener('beforeunload', function() {{ 
        console.log('Closing SSE connection');
        es.close(); 
    }});
}})();
"#, gid = gid);

    Ok(layout("Play Poker", html! {
        (fragment)
        script { (PreEscaped(sse_script)) }
    }))
}

// ============================================================================
// Play game fragment (HTMX re-fetch target for SSE updates)
// ============================================================================

#[get("/play_game_fragment?<game_id>")]
async fn play_game_fragment(
    game_id: &str,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, rocket::http::Status> {
    let session = get_session(cookies)
        .ok_or(rocket::http::Status::Unauthorized)?;
    let session_owned = session.clone();

    match client.get_game(game_id, &session_owned.user_id).await {
        Ok(s) => Ok(render_game_fragment(game_id, &session_owned, &s)),
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

