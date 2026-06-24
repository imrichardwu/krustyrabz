use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::State;
use std::sync::Arc;
use maud::{html, Markup};

use crate::{HxRedirect, SessionCache, set_session_cookie};
use crate::authentication::register_helper;

#[derive(rocket::form::FromForm)]
pub struct SignUpRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[get("/register_form")]
pub async fn register_form() -> Markup {
    html! {
        div class="bg-gray-900 rounded-xl p-8 w-full max-w-sm shadow-2xl" {
            div class="flex justify-between items-center mb-6" {
                h2 class="text-xl font-bold" style="font-family:'Playfair Display',serif;" { "Create Account" }
                button class="text-gray-500 hover:text-white text-xl leading-none"
                    onclick="document.getElementById('register_form').close()" { "×" }
            }
            form hx-post="/register" class="flex flex-col gap-4" {
                div {
                    label class="block text-xs font-semibold uppercase tracking-widest text-gray-400 mb-1.5"
                        for="reg-username" { "Username" }
                    input type="text" name="username" id="reg-username" required
                        class="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-3.5 py-2.5 text-sm focus:outline-none focus:border-yellow-500 focus:ring-1 focus:ring-yellow-500/40" {}
                }
                div {
                    label class="block text-xs font-semibold uppercase tracking-widest text-gray-400 mb-1.5"
                        for="reg-email" { "Email" }
                    input type="email" name="email" id="reg-email" required
                        class="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-3.5 py-2.5 text-sm focus:outline-none focus:border-yellow-500 focus:ring-1 focus:ring-yellow-500/40" {}
                }
                div {
                    label class="block text-xs font-semibold uppercase tracking-widest text-gray-400 mb-1.5"
                        for="reg-password" { "Password" }
                    input type="password" name="password" id="reg-password" required minlength="6"
                        class="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-3.5 py-2.5 text-sm focus:outline-none focus:border-yellow-500 focus:ring-1 focus:ring-yellow-500/40" {}
                }
                button type="submit"
                    class="w-full bg-yellow-600 text-black font-bold py-2.5 rounded-lg hover:bg-yellow-500 transition-colors mt-1" {
                    "Register"
                }
            }
        }
    }
}

#[post("/register", data = "<sign_up>")]
pub async fn register(
    sign_up: Form<SignUpRequest>,
    state: &State<Arc<SessionCache>>,
    cookies: &CookieJar<'_>,
) -> HxRedirect {
    match register_helper(&sign_up.email, &sign_up.username, &sign_up.password).await {
        Ok(auth_session) => {
            set_session_cookie(cookies, state, auth_session);
            HxRedirect::to("/main_menu")
        }
        Err(_) => HxRedirect::to("/"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![register_form, register]
}
