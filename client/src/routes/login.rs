use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::State;
use rocket::response::Redirect;
use std::sync::Arc;
use maud::{html, Markup};

use crate::{SessionCache, set_session_cookie, layout};
use crate::authentication::login_helper;

#[derive(rocket::form::FromForm)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[get("/login_page")]
pub async fn login_page() -> Markup {
    layout("Sign In — Poker", html! {
        div class="min-h-screen flex items-center justify-center px-4" style="background:#0f1117;" {
            div class="w-full max-w-md" {
                // Header
                div class="text-center mb-8" {
                    div class="inline-block mb-4 text-5xl" style="color:#42b883;" { "♠" }
                    h1 class="text-3xl font-bold mb-2" style="font-family:'Playfair Display',serif; color:white;" {
                        "Welcome Back"
                    }
                    p class="text-sm" style="color:#7a8fa6;" { "Sign in to continue playing" }
                }

                // Login form
                div class="rounded-2xl p-8" style="background:#1a2332; border:1px solid #2d3a4a;" {
                    form action="/login" method="post" class="flex flex-col gap-5" {
                        div {
                            label class="block text-xs font-semibold uppercase tracking-widest mb-1.5" 
                                style="color:#7a8fa6;" for="login-email" { "Email" }
                            input type="email" name="email" id="login-email" required
                                class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                                style="background:#0f1117; border:1px solid #2d3a4a; color:white;"
                                placeholder="your@email.com" {}
                        }
                        div {
                            label class="block text-xs font-semibold uppercase tracking-widest mb-1.5"
                                style="color:#7a8fa6;" for="login-password" { "Password" }
                            input type="password" name="password" id="login-password" required
                                class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                                style="background:#0f1117; border:1px solid #2d3a4a; color:white;"
                                placeholder="••••••••" {}
                        }
                        button type="submit"
                            class="w-full font-bold py-3 rounded-lg transition-colors mt-2"
                            style="background:#42b883; color:#0f1117;"
                            onmouseover="this.style.background='#33a070'"
                            onmouseout="this.style.background='#42b883'" {
                            "Sign In"
                        }
                    }
                    
                    div class="mt-6 text-center text-sm" style="color:#7a8fa6;" {
                        "Don't have an account? "
                        a href="/register_page" class="font-semibold transition-colors"
                            style="color:#42b883;"
                            onmouseover="this.style.color='#33a070'"
                            onmouseout="this.style.color='#42b883'" {
                            "Register here"
                        }
                    }
                }
            }
        }
    })
}

#[post("/login", data = "<login_req>")]
pub async fn login(
    login_req: Form<LoginRequest>,
    state: &State<Arc<SessionCache>>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    match login_helper("", &login_req.password, &login_req.email).await {
        Ok(auth_session) => {
            set_session_cookie(cookies, state, auth_session);
            Redirect::to("/main_menu")
        }
        Err(_) => Redirect::to("/login_page"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![login_page, login]
}
