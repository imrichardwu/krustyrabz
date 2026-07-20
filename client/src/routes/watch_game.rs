use maud::{Markup, html};
use poker_core::GameStatus;
use rocket::State;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;

use crate::api::PokerClient;
use crate::{HxRedirect, get_session};

pub async fn watch_game_fragment(client: &PokerClient, user_id: &str) -> Markup {
    match client.list_games().await {
        Ok(response) => {
            let watchable: Vec<_> = response
                .games
                .iter()
                .filter(|g| g.status != GameStatus::Finished)
                .collect();
            html! {
                div class="w-full max-w-3xl" {
                    h2 class="text-2xl font-bold mb-6" style="color:#42b883;" { "Spectate a Table" }
                    @if watchable.is_empty() {
                        div class="text-center py-16 rounded-xl" style="border:1px solid #2d3a4a; color:#4a5568;" {
                            div class="text-5xl mb-4" { "N/A" }
                            p { "No active games to spectate." }
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
                                    @for game in watchable.iter() {
                                        @let (s_txt, s_color) = match game.status {
                                            GameStatus::WaitingForPlayers => ("Waiting",     "#f6c90e"),
                                            GameStatus::InProgress        => ("In Progress", "#42b883"),
                                            GameStatus::Finished          => ("Finished",    "#4a5568"),
                                        };
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
                                                form hx-post="/register_viewer" hx-target="body" hx-swap="none" {
                                                    input type="hidden" name="viewer_id" value=(user_id) {}
                                                    input type="hidden" name="game_id" value=(game.game_id) {}
                                                    button type="submit"
                                                        class="px-4 py-1.5 text-xs font-semibold rounded-lg cursor-pointer transition-colors"
                                                        style="background:rgba(99,179,237,0.15); color:#63b3ed; border:1px solid rgba(99,179,237,0.3);"
                                                        onmouseover="this.style.background='rgba(99,179,237,0.3)'"
                                                        onmouseout="this.style.background='rgba(99,179,237,0.15)'" {
                                                        "Watch ->"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_) => html! {
            div class="rounded-xl p-6" style="background:#1a2332; border:1px solid #2d3a4a;" {
                p style="color:#f87171;" { "Failed to load games." }
            }
        },
    }
}

#[get("/watch_game")]
pub async fn watch_game(
    client: &State<PokerClient>,
    cookies: &CookieJar<'_>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let user_id = session.user_id.clone();
    drop(session);
    Ok(watch_game_fragment(client, &user_id).await)
}

#[derive(rocket::form::FromForm)]
pub struct RegisterViewerForm {
    pub viewer_id: String,
    pub game_id: String,
}

#[post("/register_viewer", data = "<req>")]
pub async fn register_viewer(
    req: Form<RegisterViewerForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> HxRedirect {
    let _session = match get_session(cookies) {
        Some(s) => s,
        None => return HxRedirect::to("/"),
    };
    let game_id = req.game_id.clone();

    match client.register_viewer(&req.viewer_id, &game_id).await {
        Ok(_) => HxRedirect::to(format!("/spectate?game_id={}", game_id)),
        Err(_) => HxRedirect::to("/main_menu"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![watch_game, register_viewer]
}
