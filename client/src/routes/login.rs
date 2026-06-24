use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::State;
use std::sync::Arc;
use maud::{html, Markup};

use crate::{HxRedirect, SessionCache, set_session_cookie};
use crate::authentication::login_helper;

#[derive(rocket::form::FromForm)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[get("/login_form")]
pub async fn login_form() -> Markup {
    html! {
        div class="bg-gray-900 rounded-xl p-8 w-full max-w-sm shadow-2xl" {
            div class="flex justify-between items-center mb-6" {
                h2 class="text-xl font-bold" style="font-family:'Playfair Display',serif;" { "Welcome Back" }
                button class="text-gray-500 hover:text-white text-xl leading-none"
                    onclick="document.getElementById('login_form').close()" { "×" }
            }
            form hx-post="/login" class="flex flex-col gap-4" {
                div {
                    label class="block text-xs font-semibold uppercase tracking-widest text-gray-400 mb-1.5"
                        for="login-email" { "Email" }
                    input type="email" name="email" id="login-email" required
                        class="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-3.5 py-2.5 text-sm focus:outline-none focus:border-yellow-500 focus:ring-1 focus:ring-yellow-500/40" {}
                }
                div {
                    label class="block text-xs font-semibold uppercase tracking-widest text-gray-400 mb-1.5"
                        for="login-password" { "Password" }
                    input type="password" name="password" id="login-password" required
                        class="w-full bg-gray-800 border border-gray-700 text-white rounded-lg px-3.5 py-2.5 text-sm focus:outline-none focus:border-yellow-500 focus:ring-1 focus:ring-yellow-500/40" {}
                }
                button type="submit"
                    class="w-full bg-yellow-600 text-black font-bold py-2.5 rounded-lg hover:bg-yellow-500 transition-colors mt-1" {
                    "Login"
                }
            }
        }
    }
}

#[post("/login", data = "<login_req>")]
pub async fn login(
    login_req: Form<LoginRequest>,
    state: &State<Arc<SessionCache>>,
    cookies: &CookieJar<'_>,
) -> HxRedirect {
    match login_helper("", &login_req.password, &login_req.email).await {
        Ok(auth_session) => {
            set_session_cookie(cookies, state, auth_session);
            HxRedirect::to("/main_menu")
        }
        Err(_) => HxRedirect::to("/"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![login_form, login]
}
