mod api;
mod games;
mod authentication;
mod viewer;
mod player;

use crate::api::PokerClient;
use games::game_settings;
use authentication::{AuthSession, login_helper, register_helper};
use rocket::{form::Form, http::Status}; 
use rocket::http::Header;
use rocket::response::Response; 
use poker_core::{GameType, GameStatus};
use rocket::State;
use rocket::fs::FileServer;
use std::sync::Arc;
use rocket::serde::json::Json;
use serde::Deserialize;
use maud::{html, Markup, DOCTYPE, PreEscaped};
use dashmap::DashMap;

type SessionCache = DashMap<String, AuthSession>;

#[derive(rocket::form::FromForm)]
struct SignUpRequest {
    email: String,
    username: String,
    password: String,
}

#[derive(rocket::form::FromForm)]
struct LoginRequest {
    email: String,
    username: String,
    password: String,
}

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    let session_cache: SessionCache = DashMap::new();
    let client = PokerClient::localhost();

    rocket::build()
        .mount("/", routes![
            landing,
            main_menu,
            list_and_join_games_helper,
            login_form,
            register_form,
            login,
            register,
            create_new_game,
            watch_game,
            add_chips,
        ])
        .mount("/public", FileServer::from("public"))
        .manage(Arc::new(session_cache))
        .manage(client)
}

fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
            html {
                head {
                    meta charset="utf-8" {}
                    title { (title) }
                    link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital@0;1&display=swap" { }
                    script src="https://cdn.jsdelivr.net/npm/@tailwindplus/elements@1" type="module" {}
                    script src="https://cdn.tailwindcss.com" {}
                    link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/cardsjs/dist/cards.min.css" {}
                    script src="https://cdn.jsdelivr.net/npm/cardshs/dist/cards.min.js" type="text/javascript" {}
                }
                script src="https://unpkg.com/htmx.org@2.0.0" {}
                script { (PreEscaped("document.addEventListener('htmx:afterRequest',function(e){var t=e.detail.target;if(t&&t.tagName==='DIALOG')t.showModal();});")) }
                body class="bg-black text-white" style="font-family: 'Roboto', sans-serif;" {
                    header {  {}
                        main style="mt-500 mb-500" { (content) }
                    }
                }
            }
    }
}

fn close_btn(target: &str) -> Markup {
    html! {
        button class="mb-8 px-2 py-1 bg-black text-white rounded"
            onclick=(format!("document.getElementById('{}').close()", target)) { "X" }
    }
}

fn action_btn(target: &str, img: &str) -> Markup {
    html! {
        button class="mb-8 px-2 py-1 bg-black text-white rounded"
            onclick=(format!("document.getElementByid('{}').click()", target)) { img src=(format!("../public/{}", img)) {} }
    }
}

fn modal_content(title: &str, elId: &str, content: Markup) -> Markup {
    html! {
        (close_btn(elId))
            h2 class="text-lg font-bold mb-4" { (title) }
        p { (content) }
    }
}

fn card(title: &str, ctls: Vec<Markup>) -> Markup {
    html! {
        h1 { (title) }
        div class="grid grid-cols-1 gap-4 mt-10" {
            @for ctl in ctls {
                (ctl)
            }
        }
    }
}

#[get("/main_menu?<id>")]
async fn main_menu(id: &str, state: &State<Arc<SessionCache>>) -> Markup {
    //need to verify session
    let verified = true;
    layout("Main Menu", html! {
        dialog id="list-and-join-games" class="rounded p-6 backdrop:bg-black/50" { }
        dialog id="create-new-game" class="rounded p-6 backdrop:bg-black/50" { }
        dialog id="watch-game" class="rounded p-6 backdrop:bg-black/50" { }
        dialog id="add-chips" class="rounded p-6 backdrop:bg-black/50" { }

        div class="hand hhand-compact active-hand" {
            form hx-get="/list_and_join_games" hx-target="#list-and-join-games" hx-swap="innerHTML" class="pb-3 text-right" {
                button type="submit" name="card" value="AS" {
                    img class="card" src="cards/AS.svg" { }
                }
            }
            form hx-get="/create_new_game" hx-target="#create-new-game" hx-swap="innerHTML" class="pb-3 text-right" {
                button type="submit" name="card" value="KS" {
                    img class="card" src="cards/KS.svg" { }
                }
            }
            form hx-get="/watch_game" hx-target="#watch-game" hx-swap="innerHTML" class="pb-3 text-right" {
                button type="submit" name="card" value="QS" {
                    img class="card" src="cards/QS.svg" { }
                }
            }
            form hx-get="/add_chips" hx-target="#add-chips" hx-swap="innerHTML" class="pb-3 text-right" {
                button type="submit" name="card" value="JS" {
                    img class="card" src="cards/JS.svg" { }
                }
            }
            form hx-get="/logout" hx-target="#" hx-swap="innerHTML" class="pb-3 text-right" {
                button type="submit" name="card" value="10S" {
                    img class="card" src="cards/10S.svg" {}
                }
            }
            form hx-get="/exit" hx-target="#" hx-swap="innerHTML" class="pb-3 text-right" {
                button type="submit" name="card" value="9H" {
                    img class="card" src="cards/9H.svg" {}
                }
            }
        }
    })
}

#[get("/list_and_join_games?<id>")]
async fn list_and_join_games_helper(client: &State<PokerClient>, id: String, state: &State<Arc<SessionCache>>) -> Markup {

    let can_join: Markup = html! {
        form hx-post="/join_and_play_game" hx-target="#" hx-swap="innerHTML" class="pb-3 text-right" {
            button type="submit" name="join" value="join" class="text-green" { "PLAY NOW" }
        }
    };

    let cant_join: Markup = html! {
        span class="text-red" { "GAME FULL" }
    };

    // List all games
    match client.list_games().await {
        Ok(response) => {
            html! {
                h1 { "AVAILABLE GAMES" }
                table {
                    thead {
                        tr {
                            th { "Game ID" }
                            th { "Type"    }
                            th { "Players" }
                            th { "Status"  }
                            th { "Join"    }
                        }
                    }
                    tbody {
                        @for game in response.games.iter() {
                            @let status = match game.status {
                                GameStatus::WaitingForPlayers => "Waiting",
                                GameStatus::InProgress => "In Progress",
                                GameStatus::Finished => "Finished",
                            };
                            tr {
                                td { (game.game_id) }
                                td { (game.game_type) }
                                td { (game.player_count) }
                                td { (status) }
                                td {
                                    @if game.player_count < 5 {
                                        (can_join)
                                    } @else {
                                        (cant_join)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_) => html! { p { "Failed to load games." } }
    }
}

#[get("/register_form")]
async fn register_form() -> Markup {
    let content = html! {
        form hx-post="/register"  {
            div.row {
                label for="username" { "Username:" }
                input type="text" name="username" id="username" required {}
            }
            div.row { 
                label for="email" { "Email:" }
                input type="email" name="email" id="email" required {}
            }
            div.row { 
                label for="password" { "Password:" }
                input type="password" name="password" id="password" required {}
            }
            div.row { 
                button type="submit" { "Submit" }
            }
        }
    };

    modal_content("REGISTER", "register_form", content)
}

#[get("/login_form")]
async fn login_form() -> Markup {
    let content = html! {
        form hx-post="/login" {
            label for="username" { "Username:" }
            input type="text" name="username" id="username" required {}

            label for="email" { "Email:" }
            input type="text" name="email" id="email" required { } 

            label for="password" { "Password:" }
            input type="password" name="password" id="password" required {}

            button type="submit" { "Submit" }
        }
    };

    modal_content("LOGIN", "login_form", content)
}

#[derive(Responder)] 
struct HxRedirect { 
    inner: String, 
    header: Header<'static>, 
}

impl HxRedirect { 
    fn to(url: impl Into<String>) -> Self { 
        HxRedirect { 
            inner: String::new(), 
            header: Header::new("HX-Redirect", url.into()), 
        }
    }
}

#[post("/register", data = "<sign_up>")]
async fn register(sign_up: Form<SignUpRequest>, state: &State<Arc<SessionCache>>) -> HxRedirect  {
    match register_helper(&sign_up.email, &sign_up.username, &sign_up.password).await {
        Ok(auth_session) => {
            state.insert(auth_session.user_id.clone(), auth_session);
           HxRedirect::to("/main_menu")
        },
        Err(_) => { 
            HxRedirect::to("/")
        },
    }
}

#[post("/login", data = "<login_req>")]
async fn login(login_req: Form<LoginRequest>, state: &State<Arc<SessionCache>>) -> HxRedirect {
    match login_helper(&login_req.username, &login_req.password, &login_req.email).await {
        Ok(auth_session) => {
            state.insert(auth_session.user_id.clone(), auth_session);
            HxRedirect::to("/main_menu")
        },
        Err(_) => HxRedirect::to("/")
    }
}


#[get("/")]
async fn landing() -> Markup {
    layout("", html! {
        dialog id="register_form" class="rounded p-6 backdrop:bg-black/50" {}
        dialog id="login_form" class="rounded p-6 backdrop:bg-black/50" {}
        div class="h1 text-center" style="font-family: 'Playfair Display', serif; color: #B8860B;" { "PLAY POKER: WIN BIG OR GO BROKE TRYING!" }
        div class="grid grid-cols-2 gap-0" { 
            div class="mt-20" { form hx-get="/register_form" hx-target="#register_form" hx-swap="innerHTML" class="pb-3 text-right" {
            button type="submit"
                style="font-family: 'Playfair Display', serif; color: #B8860B; border: 2px solid #B8860B;" class="px-4 py-2 bg-black text-yellow border border-gray-300 rounded hover:bg-gray-600 transition-colors"
                { 
                    img src="public/RED_BACK.svg" { "REGISTER" }
                }
        } } 
            div class="text-left justify-self-start mt-20" { form hx-get="/login_form" hx-target="#login_form" hx-swap="innerHTML" class="pb-3 text-right" {
            button type="submit" style="font-family: 'Playfair Display', serif; color: #B8860B; border: 2px solid #B8860B;" class="px-4 py-2 bg-black text-yellow border border-gray-300 rounded hover:bg-gray-600 transition-colors"
            {
                img src="public/BLUE_BACK.svg" { "LOGIN" } }
        } }
        }
        div class="h1 mt-20 text-center" style="font-family: 'Playfair Display', serif; color: #B8860B;" {"PLEASE PLAY SENSIBLY! 80% of gamblers quit just before they make it big. Never stop gambling! You can do it!" } 
    })
}


#[get("/create_new_game?<id>")]
async fn create_new_game(id: &str) -> Markup {
    layout("Create Game", html! {
        div class="max-w-sm" {
            h2 class="font-serif text-3xl font-bold mb-8 text-white" { "Create a Room" }
            div class="bg-zinc-900 border border-zinc-800 rounded-xl p-8" {
                form action="/start_game" method="post" {
                    input type="hidden" name="id" value=(id) {}
                    div class="mb-5" {
                        label for="game_type" class="block text-xs font-semibold tracking-widest uppercase text-zinc-400 mb-1.5" { "Game Variant" }
                        select name="game_type" id="game_type"
                            class="w-full bg-zinc-800 border border-zinc-700 text-white rounded-lg px-3.5 py-2.5 text-sm appearance-none cursor-pointer focus:outline-none focus:border-emerald-400 focus:ring-2 focus:ring-emerald-400/20" {
                            option value="FiveCardDraw"  { "Five Card Draw" }
                            option value="SevenCardStud" { "Seven Card Stud" }
                            option value="TexasHoldEm"   { "Texas Hold'Em" }
                        }
                    }
                    button type="submit"
                        class="w-full bg-emerald-400 text-emerald-950 font-bold px-7 py-3 rounded-lg text-base hover:bg-emerald-300 transition-colors cursor-pointer border-0" {
                        "Create Room"
                    }
                }
            }
        }
    })
}

#[get("/watch_game?<id>")]
async fn watch_game(id: String, client: &State<PokerClient>) -> Markup {
    let back = format!("/main_menu?id={}", id);
    match client.list_games().await {
        Ok(response) => {
            let watchable: Vec<_> = response.games.iter()
                .filter(|g| g.status != GameStatus::Finished).collect();
            layout("Spectate", html! {
                h2 class="font-serif text-3xl font-bold mb-8 text-white" { "Spectate a Game" }
                @if watchable.is_empty() {
                    div class="bg-amber-900/20 border border-amber-700/40 text-amber-400 rounded-xl p-10 text-center" {
                        div class="text-4xl mb-3" { "♦" }
                        div class="font-semibold" { "No active games to watch" }
                    }
                } @else {
                    div class="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden" {
                        table class="w-full border-collapse text-sm" {
                            thead {
                                tr {
                                    th class="text-left px-4 py-3 text-xs font-bold tracking-widest uppercase text-zinc-600 border-b border-zinc-800" { "Game ID" }
                                    th class="text-left px-4 py-3 text-xs font-bold tracking-widest uppercase text-zinc-600 border-b border-zinc-800" { "Type" }
                                    th class="text-left px-4 py-3 text-xs font-bold tracking-widest uppercase text-zinc-600 border-b border-zinc-800" { "Players" }
                                    th class="text-left px-4 py-3 text-xs font-bold tracking-widest uppercase text-zinc-600 border-b border-zinc-800" { "Status" }
                                    th class="px-4 py-3 border-b border-zinc-800" { "" }
                                }
                            }
                            tbody {
                                @for game in watchable.iter() {
                                    @let (s_txt, s_cls) = match game.status {
                                        GameStatus::WaitingForPlayers => ("Waiting",     "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold bg-amber-900/30 text-amber-400"),
                                        GameStatus::InProgress        => ("In Progress", "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold bg-emerald-900/30 text-emerald-400"),
                                        GameStatus::Finished          => ("Finished",    "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold bg-zinc-800 text-zinc-500"),
                                    };
                                    tr class="hover:bg-zinc-800/50 transition-colors" {
                                        td class="px-4 py-3 border-b border-zinc-800" {
                                            span class="font-mono text-xs text-zinc-500" { (game.game_id) }
                                        }
                                        td class="px-4 py-3 border-b border-zinc-800 text-white" { (game.game_type) }
                                        td class="px-4 py-3 border-b border-zinc-800 text-zinc-400" { (game.player_count) "/" (game.max_players) }
                                        td class="px-4 py-3 border-b border-zinc-800" {
                                            span class=(s_cls) { (s_txt) }
                                        }
                                        td class="px-4 py-3 border-b border-zinc-800" {
                                            form action="/register_viewer" method="post" {
                                                input type="hidden" name="id" value=(id) {}
                                                input type="hidden" name="game_id" value=(game.game_id) {}
                                                button type="submit"
                                                    class="inline-flex items-center px-3 py-1.5 text-xs font-semibold rounded-lg bg-blue-900/30 text-blue-400 border border-blue-800/50 hover:bg-blue-900/50 transition-colors cursor-pointer" {
                                                    "Watch"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })
        }
        Err(e) => layout("Spectate", html! {
            a href=(back) class="inline-flex items-center px-4 py-2 text-sm text-zinc-400 border border-zinc-700 rounded-lg hover:bg-zinc-800 hover:text-white transition-colors mb-4" { "← Lobby" }
            div class="bg-red-900/20 border border-red-700/40 text-pink-400 rounded-lg p-3 text-sm" { (e) }
        }),
    }
}

#[get("/add_chips?<id>")]
async fn add_chips(id: &str) -> Markup {
    layout("Add Chips", html! {
        div class="max-w-sm" {
            h2 class="font-serif text-3xl font-bold mb-8 text-white" { "Add Chips" }
            div class="bg-zinc-900 border border-zinc-800 rounded-xl p-8" {
                form action="/chips" method="post" {
                    input type="hidden" name="id" value=(id) {}

                    div class="mb-5" {
                        label class="block text-xs font-semibold tracking-widest uppercase text-zinc-400 mb-1.5" { "Quick Select" }
                        div class="flex gap-2" {
                            button type="button"
                                class="flex-1 bg-zinc-800 border border-zinc-600 text-white rounded-lg py-2 font-mono text-sm font-bold cursor-pointer hover:border-emerald-400 hover:text-emerald-400 hover:bg-emerald-900/20 transition-all"
                                onclick="document.getElementById('amount').value='1000'" { "1,000" }
                            button type="button"
                                class="flex-1 bg-zinc-800 border border-zinc-600 text-white rounded-lg py-2 font-mono text-sm font-bold cursor-pointer hover:border-emerald-400 hover:text-emerald-400 hover:bg-emerald-900/20 transition-all"
                                onclick="document.getElementById('amount').value='5000'" { "5,000" }
                            button type="button"
                                class="flex-1 bg-zinc-800 border border-zinc-600 text-white rounded-lg py-2 font-mono text-sm font-bold cursor-pointer hover:border-emerald-400 hover:text-emerald-400 hover:bg-emerald-900/20 transition-all"
                                onclick="document.getElementById('amount').value='10000'" { "10,000" }
                        }
                    }

                    div class="mb-5" {
                        label for="amount" class="block text-xs font-semibold tracking-widest uppercase text-zinc-400 mb-1.5" { "Custom Amount" }
                        input type="number" name="amount" id="amount" required min="1"
                            class="w-full bg-zinc-800 border border-zinc-700 text-white rounded-lg px-3.5 py-2.5 text-sm focus:outline-none focus:border-emerald-400 focus:ring-2 focus:ring-emerald-400/20 placeholder-zinc-600"
                            placeholder="Enter amount" {}
                    }

                    button type="submit"
                        class="w-full bg-emerald-400 text-emerald-950 font-bold px-7 py-3 rounded-lg text-base hover:bg-emerald-300 transition-colors cursor-pointer border-0 mb-4" {
                        "Add Chips"
                    }

                    p class="text-xs text-zinc-600 text-center" {
                        "Note: You cannot add chips while in an active game."
                    }
                }
            }
        }
    })
}
