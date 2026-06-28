use rocket::post;
use rocket::http::CookieJar;
use rocket::State;
use crate::{get_session, HxRedirect};
use crate::api::PokerClient;

/// POST /game/leave - Leave the current game
#[post("/game/leave?<game_id>")]
pub async fn leave_game(
    game_id: String,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> HxRedirect {
    let session = match get_session(cookies) {
        Some(s) => s,
        None => return HxRedirect::to("/"),
    };

    let player_id = session.user_id.clone();
    drop(session);

    match client.leave_game(&player_id, &game_id).await {
        Ok(_) => HxRedirect::to("/main_menu"),
        Err(_) => HxRedirect::to("/main_menu"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![leave_game]
}
