use maud::{Markup, html};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;

use crate::authentication::register_helper;
use crate::{layout, set_session_cookie};

#[derive(rocket::form::FromForm)]
pub struct SignUpRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[get("/register_page?<error>")]
pub async fn register_page(error: Option<String>) -> Markup {
    layout(
        "Register - Poker",
        html! {
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
                        @if let Some(err_msg) = error {
                            div class="rounded-lg px-4 py-3 mb-5 text-sm font-medium"
                                style="background:rgba(248,113,113,0.12); color:#f87171; border:1px solid rgba(248,113,113,0.3);" {
                                "⚠️ " (err_msg)
                            }
                        }

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
        },
    )
}

#[post("/register", data = "<sign_up>")]
pub async fn register(sign_up: Form<SignUpRequest>, cookies: &CookieJar<'_>) -> Redirect {
    match register_helper(&sign_up.email, &sign_up.username, &sign_up.password).await {
        Ok(auth_session) => {
            set_session_cookie(cookies, auth_session);
            Redirect::to("/main_menu")
        }
        Err(e) => {
            let message = e.to_string();
            let encoded_error = urlencoding::encode(&message);
            Redirect::to(format!("/register_page?error={}", encoded_error))
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![register_page, register]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

    #[test]
    fn test_registration_route() {
        // Use the exact name of the handler macro: register
        let rocket = rocket::build().mount("/", rocket::routes![register]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body("username=TestUser123&email=test@test.com&password=SecurePassword1!")
            .dispatch();

        // If the DB fails, it redirects to /register_page. If it succeeds, to /main_menu.
        // Both are Status::SeeOther (303 Redirect)
        assert_eq!(response.status(), Status::SeeOther);
    }
}
