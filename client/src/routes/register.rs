use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use maud::{html, Markup};

use crate::{set_session_cookie, layout};
use crate::authentication::register_helper;

#[derive(rocket::form::FromForm)]
pub struct SignUpRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[get("/register_page")]
pub async fn register_page() -> Markup {
    layout("Register - Poker", html! {
        div class="min-h-screen flex items-center justify-center px-4" style="background:#0f1117;" {
            div class="w-full max-w-md" {
                // Header
                div class="text-center mb-8" {
                    div class="inline-block mb-4 text-5xl" style="color:#42b883;" { "P" }
                    h1 class="text-3xl font-bold mb-2" style="color:white;" {
                        "Join The Table"
                    }
                    p class="text-sm" style="color:#7a8fa6;" { "Create your account to start playing" }
                }

                // Register form
                div class="rounded-2xl p-8" style="background:#1a2332; border:1px solid #2d3a4a;" {
                    form action="/register" method="post" class="flex flex-col gap-5" {
                        div {
                            label class="block text-xs font-semibold uppercase tracking-widest mb-1.5" 
                                style="color:#7a8fa6;" for="reg-username" { "Username" }
                            input type="text" name="username" id="reg-username" required
                                class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                                style="background:#0f1117; border:1px solid #2d3a4a; color:white;"
                                placeholder="pokerpro123" {}
                        }
                        div {
                            label class="block text-xs font-semibold uppercase tracking-widest mb-1.5"
                                style="color:#7a8fa6;" for="reg-email" { "Email" }
                            input type="email" name="email" id="reg-email" required
                                class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                                style="background:#0f1117; border:1px solid #2d3a4a; color:white;"
                                placeholder="your@email.com" {}
                        }
                        div {
                            label class="block text-xs font-semibold uppercase tracking-widest mb-1.5"
                                style="color:#7a8fa6;" for="reg-password" { "Password" }
                            input type="password" name="password" id="reg-password" required minlength="6"
                                class="w-full rounded-lg px-3.5 py-2.5 text-sm focus:outline-none"
                                style="background:#0f1117; border:1px solid #2d3a4a; color:white;"
                                placeholder="password" {}
                            p class="text-xs mt-1" style="color:#4a5568;" { "Minimum 6 characters" }
                        }
                        button type="submit"
                            class="w-full font-bold py-3 rounded-lg transition-colors mt-2"
                            style="background:#42b883; color:#0f1117;"
                            onmouseover="this.style.background='#33a070'"
                            onmouseout="this.style.background='#42b883'" {
                            "Create Account"
                        }
                    }
                    
                    div class="mt-6 text-center text-sm" style="color:#7a8fa6;" {
                        "Already have an account? "
                        a href="/login_page" class="font-semibold transition-colors"
                            style="color:#42b883;"
                            onmouseover="this.style.color='#33a070'"
                            onmouseout="this.style.color='#42b883'" {
                            "Sign in"
                        }
                    }
                }
            }
        }
    })
}

#[post("/register", data = "<sign_up>")]
pub async fn register(
    sign_up: Form<SignUpRequest>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    match register_helper(&sign_up.email, &sign_up.username, &sign_up.password).await {
        Ok(auth_session) => {
            set_session_cookie(cookies, auth_session);
            Redirect::to("/main_menu")
        }
        Err(_) => Redirect::to("/register_page"),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![register_page, register]
}
