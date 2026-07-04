use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use maud::{html, Markup};
use poker_core::GameStatus;

use crate::{get_session, HxRedirect};
use crate::api::PokerClient;

#[get("/list_and_join_games")]
pub async fn list_and_join_games(
    client: &State<PokerClient>,
    cookies: &CookieJar<'_>,
) -> Result<Markup, Redirect> {
    let _session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;

    let fragment = match client.list_games().await {
        Ok(response) => html! {
            div class="w-full max-w-3xl" {
                h2 class="text-2xl font-bold mb-6" style="color:#42b883;" { "Available Tables" }
                @if response.games.is_empty() {
                    div class="text-center py-16 rounded-xl" style="border:1px solid #2d3a4a; color:#4a5568;" {
                        div class="text-5xl mb-4" { "N/A" }
                        p { "No games available. Create one!" }
                    }
                } @else {
                    div class="overflow-x-auto rounded-xl" style="border:1px solid #2d3a4a;" {
                        table class="w-full text-sm" {
                            thead {
                                tr style="background:#1a2332; border-bottom:1px solid #2d3a4a;" {
                                    th class="text-left px-4 py-3 text-xs uppercase tracking-widest" style="color:#4a5568;" { "Game ID" }
                                    th class="text-left px-4 py-3 text-xs uppercase tracking-widest" style="color:#4a5568;" { "Type" }
                                    th class="text-left px-4 py-3 text-xs uppercase tracking-widest" style="color:#4a5568;" { "Players" }
                                    th class="text-left px-4 py-3 text-xs uppercase tracking-widest" style="color:#4a5568;" { "Status" }
                                    th class="px-4 py-3" {}
                                }
                            }
                            tbody {
                                @for game in response.games.iter() {
                                    @let (s_txt, s_color) = match game.status {
                                        GameStatus::WaitingForPlayers => ("Waiting",     "#f6c90e"),
                                        GameStatus::InProgress        => ("In Progress", "#42b883"),
                                        GameStatus::Finished          => ("Finished",    "#4a5568"),
                                    };
                                    @let can_join = game.player_count < game.max_players
                                        && game.status == GameStatus::WaitingForPlayers;
                                    tr style="border-bottom:1px solid #1e2d3d;"
                                        onmouseover="this.style.background='rgba(66,184,131,0.04)'"
                                        onmouseout="this.style.background=''" {
                                        td class="px-4 py-3" {
                                            span class="font-mono text-xs" style="color:#4a5568;" {
                                                (game.game_id.chars().take(8).collect::<String>()) "..."
                                            }
                                        }
                                        td class="px-4 py-3 font-medium" style="color:white;" { (game.game_type) }
                                        td class="px-4 py-3" style="color:#7a8fa6;" {
                                            (game.player_count) "/" (game.max_players)
                                        }
                                        td class="px-4 py-3" {
                                            span class="px-2 py-0.5 rounded-full text-xs font-semibold"
                                                style=(format!("color:{}; background:{}20;", s_color, s_color)) {
                                                (s_txt)
                                            }
                                        }
                                        td class="px-4 py-3" {
                                            @if can_join {
                                                form hx-post="/join_game" hx-target="body" hx-swap="none" {
                                                    input type="hidden" name="game_id" value=(game.game_id) {}
                                                    button type="submit"
                                                        class="px-4 py-1.5 text-xs font-semibold rounded-lg cursor-pointer transition-colors"
                                                        style="background:rgba(66,184,131,0.15); color:#42b883; border:1px solid rgba(66,184,131,0.3);"
                                                        onmouseover="this.style.background='rgba(66,184,131,0.3)'"
                                                        onmouseout="this.style.background='rgba(66,184,131,0.15)'" {
                                                        "Join ->"
                                                    }
                                                }
                                            } @else {
                                                span class="text-xs" style="color:#2d3a4a;" { "—" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(_) => html! {
            div class="rounded-xl p-6" style="background:#1a2332; border:1px solid #2d3a4a;" {
                p style="color:#f87171;" { "Failed to load games. Is the server running?" }
            }
        },
    };
    Ok(fragment)
}

#[derive(rocket::form::FromForm)]
pub struct JoinGameForm {
    pub game_id: String,
}

#[post("/join_game", data = "<req>")]
pub async fn join_game(
    req: Form<JoinGameForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> HxRedirect {
    let session = match get_session(cookies) {
        Some(s) => s,
        None => return HxRedirect::to("/"),
    };
    let user_id = session.user_id.clone();
    let username = session.username.clone();
    let game_id = req.game_id.clone();
    drop(session);

    match client.join_game(&user_id, &username, &game_id).await {
        Ok(resp) => {
            if resp.success {
                println!("Successfully joined game: {}", game_id);
                HxRedirect::to(format!("/play_game?game_id={}", game_id))
            } else {
                eprintln!("Join game failed: {}", resp.message);
                HxRedirect::to("/main_menu")
            }
        }
        Err(e) => {
            eprintln!("Join game error: {:?}", e);
            HxRedirect::to("/main_menu")
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![list_and_join_games, join_game]
}
