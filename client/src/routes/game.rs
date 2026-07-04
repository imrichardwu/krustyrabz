use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use maud::Markup;

use crate::{get_session, render_game_fragment};
use crate::api::PokerClient;

// ─── Shared form structs ──────────────────────────────────────────────────────

#[derive(rocket::form::FromForm)]
pub struct GameIdForm {
    pub game_id: String,
}

#[derive(rocket::form::FromForm)]
pub struct AmountForm {
    pub game_id: String,
    pub amount: u32,
}

#[derive(rocket::form::FromForm)]
pub struct DrawForm {
    pub game_id: String,
    pub discard_indices: Vec<usize>,
}

// ─── Routes ───────────────────────────────────────────────────────────────────

#[post("/game/start_hand", data = "<req>")]
pub async fn start_hand(
    req: Form<GameIdForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.start_hand(game_id, user_id).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/fold", data = "<req>")]
pub async fn fold(
    req: Form<GameIdForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.fold(user_id, game_id).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/check", data = "<req>")]
pub async fn check(
    req: Form<GameIdForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.check(user_id, game_id).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/call", data = "<req>")]
pub async fn call(
    req: Form<GameIdForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.call(user_id, game_id).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/bet", data = "<req>")]
pub async fn bet(
    req: Form<AmountForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.bet(user_id, game_id, req.amount).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/raise", data = "<req>")]
pub async fn raise(
    req: Form<AmountForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.raise(user_id, game_id, req.amount).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/draw", data = "<req>")]
pub async fn draw(
    req: Form<DrawForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    let new_state = match client.draw(user_id, game_id, req.discard_indices.clone()).await {
        Ok(r) if r.game_state.is_some() => r.game_state.unwrap(),
        _ => match client.get_game(game_id, user_id).await {
            Ok(s) => s,
            Err(_) => return Err(Redirect::to("/main_menu")),
        },
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

#[post("/game/sit_out", data = "<req>")]
pub async fn sit_out(
    req: Form<GameIdForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let session_owned = session.clone();
    drop(session);

    let game_id = &req.game_id;
    let user_id = &session_owned.user_id;

    // Best-effort sit-out; re-fetch state regardless
    let _ = client.sit_out_hand(user_id, game_id).await;
    let new_state = match client.get_game(game_id, user_id).await {
        Ok(s) => s,
        Err(_) => return Err(Redirect::to("/main_menu")),
    };

    Ok(render_game_fragment(game_id, &session_owned, &new_state))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![start_hand, fold, check, call, bet, raise, draw, sit_out]
}
