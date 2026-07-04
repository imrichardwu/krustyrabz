use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use maud::{html, Markup};
use poker_core::GameType;

use crate::{get_session, HxRedirect};
use crate::api::PokerClient;

#[get("/create_new_game")]
pub async fn create_new_game(
    cookies: &CookieJar<'_>,
) -> Result<Markup, Redirect> {
    let _session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let fragment = html! {
        div class="w-full max-w-md" {
            h2 class="text-2xl font-bold mb-6" style="color:#42b883;" { "Create a Table" }
            form hx-post="/start_game" hx-target="body" hx-swap="none" class="flex flex-col gap-5" {
                div {
                    label for="game_type" class="block text-xs font-semibold uppercase tracking-widest mb-1.5" style="color:#7a8fa6;" { "Game Variant" }
                    select name="game_type" id="game_type"
                        class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                        style="background:#0f1117; border:1px solid #2d3a4a; color:white;" {
                        option value="FiveCardDraw"  { "Five Card Draw" }
                        option value="SevenCardStud" { "Seven Card Stud" }
                        option value="TexasHoldEm"   { "Texas Hold'em" }
                    }
                }
                button type="submit"
                    class="w-full font-bold py-3 rounded-lg transition-colors"
                    style="background:#42b883; color:#0f1117;"
                    onmouseover="this.style.background='#33a070'"
                    onmouseout="this.style.background='#42b883'" {
                    "Create Table ->"
                }
            }
        }
    };
    Ok(fragment)
}

#[derive(rocket::form::FromForm)]
pub struct StartGameForm {
    pub game_type: String,
}

#[post("/start_game", data = "<req>")]
pub async fn start_game(
    req: Form<StartGameForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> HxRedirect {
    let session = match get_session(cookies) {
        Some(s) => s,
        None => return HxRedirect::to("/"),
    };
    let user_id = session.user_id.clone();
    let username = session.username.clone();
    drop(session);

    let game_type = match req.game_type.as_str() {
        "SevenCardStud" => GameType::SevenCardStud,
        "TexasHoldEm"   => GameType::TexasHoldEm,
        _               => GameType::FiveCardDraw,
    };

    match client.create_game(&user_id, &username, game_type).await {
        Ok(resp) if resp.success => {
            let gid = resp.game_id.unwrap_or_default();
            HxRedirect::to(format!("/play_game?game_id={}", gid))
        }
        _ => HxRedirect::to("/main_menu"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![create_new_game, start_game]
}
