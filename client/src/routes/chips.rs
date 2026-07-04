use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use maud::{html, Markup};

use crate::{get_session};
use crate::api::PokerClient;

#[get("/add_chips")]
pub async fn add_chips(
    cookies: &CookieJar<'_>,
) -> Result<Markup, Redirect> {
    let _session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    Ok(chips_form(None))
}

fn chips_form(message: Option<(String, bool)>) -> Markup {
    html! {
        div class="w-full max-w-md" {
            h2 class="text-2xl font-bold mb-2" style="color:#42b883;" { "Credit Bureau" }
            p class="text-sm mb-6" style="color:#7a8fa6;" { "Purchase chips to play at any table." }

            @if let Some((ref msg, ok)) = message {
                div class="rounded-lg px-4 py-3 mb-5 text-sm font-medium"
                    style=(if ok { "background:rgba(66,184,131,0.12); color:#42b883; border:1px solid rgba(66,184,131,0.3);" }
                           else  { "background:rgba(248,113,113,0.12); color:#f87171; border:1px solid rgba(248,113,113,0.3);" }) {
                    (msg)
                }
            }

            form id="chips-form" class="flex flex-col gap-5" {
                div {
                    label class="block text-xs font-semibold uppercase tracking-widest mb-2" style="color:#7a8fa6;" { "Quick Select" }
                    div class="flex gap-2" {
                        @for amount in ["1000", "5000", "10000"] {
                            button type="button"
                                class="flex-1 rounded-lg py-2 font-mono text-sm font-bold transition-all cursor-pointer"
                                style="background:#1a2332; border:1px solid #2d3a4a; color:white;"
                                onmouseover="this.style.borderColor='#42b883'; this.style.color='#42b883';"
                                onmouseout="this.style.borderColor='#2d3a4a'; this.style.color='white';"
                                onclick=(format!("document.getElementById('chips-amount').value='{}'", amount)) {
                                (amount.chars().rev().collect::<Vec<_>>()
                                    .chunks(3)
                                    .map(|c| c.iter().rev().collect::<String>())
                                    .rev()
                                    .collect::<Vec<_>>()
                                    .join(","))
                            }
                        }
                    }
                }
                div {
                    label for="chips-amount" class="block text-xs font-semibold uppercase tracking-widest mb-1.5" style="color:#7a8fa6;" { "Amount" }
                    input type="number" name="amount" id="chips-amount" required min="1"
                        class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                        style="background:#0f1117; border:1px solid #2d3a4a; color:white;"
                        placeholder="Enter amount" {}
                }
                div class="flex gap-3" {
                    button type="button"
                        hx-post="/chips"
                        hx-target="#main-content"
                        hx-swap="innerHTML"
                        hx-include="#chips-form"
                        class="flex-1 font-bold py-3 rounded-lg transition-colors cursor-pointer"
                        style="background:#7c3aed; color:white;"
                        onmouseover="this.style.background='#6d28d9'"
                        onmouseout="this.style.background='#7c3aed'" {
                        "Deposit"
                    }
                    button type="button"
                        hx-post="/withdrawchips"
                        hx-target="#main-content"
                        hx-swap="innerHTML"
                        hx-include="#chips-form"
                        class="flex-1 font-bold py-3 rounded-lg transition-colors cursor-pointer"
                        style="background:#1a2332; color:#f6c90e; border:1px solid #f6c90e;"
                        onmouseover="this.style.background='rgba(246,201,14,0.12)'"
                        onmouseout="this.style.background='#1a2332'" {
                        "Withdraw"
                    }
                }
                p class="text-xs text-center" style="color:#2d3a4a;" { "Cannot add or withdraw chips during an active game." }
            }
        }
    }
}

#[derive(rocket::form::FromForm)]
pub struct AddChipsForm {
    pub amount: u32,
}

#[derive(rocket::form::FromForm)]
pub struct WithdrawChipsForm {
    pub amount: u32,
}

#[post("/chips", data = "<req>")]
pub async fn add_chips_post(
    req: Form<AddChipsForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let user_id = session.user_id.clone();
    drop(session);

    match client.add_chips(&user_id, req.amount).await {
        Ok(resp) if resp.success => Ok(chips_form(Some((
            format!("Successfully deposited {} chips to your account!", req.amount),
            true,
        )))),
        Ok(resp) => Ok(chips_form(Some((resp.message, false)))),
        Err(_) => Ok(chips_form(Some(("Failed to add chips. Please try again.".to_string(), false)))),
    }
}

#[post("/withdrawchips", data = "<req>")]
pub async fn withdraw_chips_post(
    req: Form<WithdrawChipsForm>,
    cookies: &CookieJar<'_>,
    client: &State<PokerClient>,
) -> Result<Markup, Redirect> {
    let session = get_session(cookies).ok_or_else(|| Redirect::to("/"))?;
    let user_id = session.user_id.clone();
    drop(session);

    match client.withdraw_chips(&user_id, req.amount).await {
        Ok(resp) if resp.success => Ok(chips_form(Some((
            format!("Successfully withdrew {} chips. New balance: {}.", resp.chips_withdrawn, resp.new_balance),
            true,
        )))),
        Ok(resp) => Ok(chips_form(Some((resp.message, false)))),
        Err(_) => Ok(chips_form(Some(("Failed to withdraw chips. Please try again.".to_string(), false)))),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![add_chips, add_chips_post, withdraw_chips_post]
}
