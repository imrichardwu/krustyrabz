use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use maud::{html, Markup};

use crate::{set_session_cookie, get_session, layout};
use crate::authentication::login_helper;

#[derive(rocket::form::FromForm)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[get("/login_page?<error>")]
pub async fn login_page(error: Option<String>, cookies: &CookieJar<'_>) -> Result<Markup, Redirect> {
    if get_session(cookies).is_some() {
        return Err(Redirect::to("/main_menu"));
    }
    Ok(layout("Sign In - Poker", html! {
        div class="min-h-screen flex items-center justify-center px-4" style="background:#0f1117;" {
            div class="w-full max-w-md" {
                // Header
                div class="text-center mb-8" {
                    div class="inline-block mb-4 text-5xl" style="color:#42b883;" { "P" }
                    h1 class="text-3xl font-bold mb-2" style="color:white;" {
                        "Welcome Back"
                    }
                    p class="text-sm" style="color:#7a8fa6;" { "Sign in to continue playing" }
                }

                // Login form
                div class="rounded-2xl p-8" style="background:#1a2332; border:1px solid #2d3a4a;" {
                    @if let Some(err_msg) = error {
                        div class="rounded-lg px-4 py-3 mb-5 text-sm font-medium" 
                            style="background:rgba(248,113,113,0.12); color:#f87171; border:1px solid rgba(248,113,113,0.3);" {
                            "⚠️ " (err_msg)
                        }
                    }

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
                                placeholder="password" {}
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
    }))
}

#[post("/login", data = "<login_req>")]
pub async fn login(
    login_req: Form<LoginRequest>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    match login_helper("", &login_req.password, &login_req.email).await {
        Ok(auth_session) => {
            set_session_cookie(cookies, auth_session);
            Redirect::to("/main_menu")
        }
        Err(e) => {
            let encoded_error = urlencoding::encode(&e);
            Redirect::to(format!("/login_page?error={}", encoded_error))
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![login_page, login]
}
