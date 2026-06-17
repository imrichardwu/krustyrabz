mod api;
mod games;
mod authentication;
mod viewer;
mod player;

use crate::api::PokerClient;
use games::game_settings;
use authentication::{login, register, AuthSession};
use viewer::watch_game;
use poker_core::{GameType, GameStatus};
use client::{PokerClient, read_input};
use rocket::State; 
use rocket::fs::FileServer; 
use std::sync::Arc; 
use rocket::serde::json::Json; 
use serde::Deserialize; 
use maud::{html, Markup, DOCTYPE, Render}; 

type SessionCache = DashMap<String, AuthSession>;

#[macro_use] extern crate rocket; 

#[launch] 
fn rocket() -> _ {
    let session_cache: SessionCache = DashMap::new();
    let client = PokerClient::new(); 

    rocket::build() 
        .mount("/", routes![watch_game, 
            list_and_join_games, 
            add_chips,
            create_new_game])
        .mount("/public", FileServer::from("../public"))
        .manage(Arc::new(session_cache))
        .manage(client)
}

fn layout(title: &str, content: Markup) -> Markup { 
    html! { 
        (DOCTYPE)
            html {
                head { 
                    meta charset="utf-8" 
                    title { (title) } 
                    link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;700&display=swap" {} 
                    script src="https://cdn.jsdelivr.net/npm/@tailwindplus/elements@1" type="module" {} 
                    script src="https://cdn.tailwindcss.com" {} 
                    link rel="stylesheet" type="text/css" href="https://unpkg.com/cardsJS/dist/cards.min.css" { } 
                    script src="https://unpkg.com/cardsJS/dist/cards.min.js" type="text/javascript" { } 
                }
                script src="https://unpkg.com/htmx.org@2.0.0" {} 
                body class = "bg-black text-white" style="font-family: 'Roboto', sans-serif;" {
                    header { img src="../public/logo.png"; 
                        main style="mt-500 mb-500" { (content) } 
                }
            }
        }
    }
}

fn close_btn(target: &str) -> Markup { 
    html! { 
        button class="mb-8 px-2 py-1 bg-black text-white rounded" 
            onclick=(format!("document.getElementById('{}').close()", target)) { "X" } 
    }
}

fn action_btn(target: &str, img: &str) -> Markup { 
    html! { 
        button class="mb-8 px-2 py-1 bg-black text-white rounded" 
            onclick=(format!("document.getElementByid('{}').click()", target)) { img src= format!("../public/{}", img) } 
    }
}

fn modal_content(title: &str, elId: &str, content: Markup) -> Markup {     
    
    html! { 
        (close_btn(elId)) 
            h2 class={"text-lg font-bold mb-4 "} { (title) } 
        p { (content) } 
    }
}

fn card(title: &str, ctls: Vec<Markup>) -> Markup { 
    html! { 
        h1 { (title) } 
        div class="grid grid-cols-1 gap-4 mt-10" { {}
            {
                @for ctl in ctls {
                    (ctl)
                }
            }
        }
    }
}

#[get("/main_menu?<id>")]
async fn main_menu(id: &str, state: &State<Arc<SessionCache>>) { 
    //need to verify session 
    let verified = true; 
    html! { 
        dialog id="list-and-join-games" class="rounded p-6 backdrop:bg-black/50" { } 
        dialog id="create-new-game" class="rounded p-6 backdrop:bg-black/50" { } 
        dialog id="watch-game" class="rounded p-6 backdrop:bg-black/50" { } 
        dialog id="add-chips" class="rounded p-6 backdrop:bg-black/50" { } 

        div class="hand hhand-compact active-hand" {  
            form hx-get="/list_and_join_games" hx-target="#list-and-join-games" hx-swap="innerHTML" hx-on::after_request="document.getElementById('list-and-join-games').showModal();" class="pb-3 text-right" { 
                button type="submit" name="card" value="AS" { 
                    img class="card" src="cards/AS.svg" { } 
                }
            }
            form hx-get="/create_new_game" hx-target="#create-new-game" hx-swap="innerHTML" hx-on::after-request="document.getElementById('create-new-game').showModal();" class="pb-3 text-right" { 
                button type="submit" name="card" value="KS" { 
                    img class="card" src="cards/KS.svg" { } 
                }
            }
            form hx-get="/watch_game" hx-target="#watch-game" hx-swap="innerHTML" hx-on::after-request="document.getElementById('watch-game').showModal();" class="pb-3 text-right" { 
                button type="submit" name="card" value="QS" { 
                    img class="card" src="cards/QS.svg" { } 
                }
            }
            form hx-get="/add_chips" hx-target="#add-chips" hx-swap="innerHTML" hx-on::after-request="document.getElementById('add-chips').showModal();" class="pb-3 text-right" { 
                button type="submit" name="card" value="JS" { 
                    img class="card" src="cards/JS.svg" { }
                }
            }
            form hx-get="/logout" hx-target="#" hx-swap="innerHTML" class="pb-3 text-right" { 
                button type="submit" name="card" value="10S" { 
                    img class="card" src="cards/10S.svg" 
                }
            }
            form hx-get="/exit" hx-target="#" hx-swap="innerHTML" class="pb-3 text-right" { 
                button type="submit" name="card" value="9H" { 
                    img class="card" src="cards/9H.svg" 
            }
        }
    }
} 


#[get("/list_and_join_games?<id>")] 
async fn list_and_join_games_helper(client: &State<PokerClient>, id: String, state: &State<Arc<SessionCache>>) -> Markup {

    let can_join: Markup = html! { 
        form hx-post="/join_and_play_game" hx-target="#" hx-swap="innerHTML" class="pb-3 text-right" { 
            button type="submit" name="join" value="join" class="text-green" { "PLAY NOW" }        
        }
}; 

    let cant_join: Markup = html! { 
        text class="text-red" { "GAME FULL" } 
    }; 

    // List all games
    match client.list_games().await {
        Ok(response) => {

            html! { 
                if response.games.is_empty() {
                    let msg: Markup = html! { p { "No games available" } }; 
                    return msg; 
                }

                h1 { "AVAILABLE GAMES" } 
                table { 
                    thead { 
                        tr { 
                            th { "Game ID" } 
                            th { "Type"    } 
                            th { "Players" }
                            th { "Status"  } 
                            th { "Join"    } 
                        }
                    }
                    tbody { 
                        @for (idx, game) in response.games.iter().enumerate() { 
                            let status = match game.status { 
                                GameStatus::WaitingForPlayers => "Waiting", 
                                GameStatus::InProgress => "In Progress", 
                                GameStatus::Finished => "Finished", 
                            }; 
                            tr { 
                                td { (game.game_id) }
                                td { (game.game_type) }
                                td { (game.player_count) }                                 
                                td { (status) } 
                                td {  
                                    @if game.player_count < 5 {
                                        (can_join)
                                    }
                                    else { 
                                        (cant_join) 
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
            
 async fn create_new_game(client: &PokerClient, session: &AuthSession) {
    println!("\n=== Create New Game ===");
    println!("1. Five Card Draw (supported)");
    println!("2. Seven Card Stud (not yet supported)");
    println!("3. Texas Hold'em (not yet supported)");
    println!("4. Back to Main Menu");
    
    let choice = read_input("Choose game type: ");
    
    let game_type = match choice.trim() {
        "1" => GameType::FiveCardDraw,
        "2" => {
            println!("Only Five Card Draw is supported for now.");
            return;
        }
        "3" => {
            println!("Only Five Card Draw is supported for now.");
            return;
        }
        "4" => {
            println!("Returning to main menu...");
            return;
        }
        _ => {
            println!("Invalid choice.");
            return;
        }
    };
    
    if let Err(e) = game_settings::create_and_play_game(client, session, game_type).await {
        println!("Error creating game: {}", e);
    }
}

#[get("/register_form")] 
async fn register_form() { 
    let content = html! { 
        form hx-post="/register" hx-target="#" hx-swap="innerHTML" {
            label for="username" { "Username:" } 
            input type="text" name="username" id="username" required; 

            label for="email" { "Email:" } 
            input type="email" name="email" id="email" required; 
            
            label for="password" { "Password:" }
            input type="password" name="password" id="password" required; 

            button type="submit" { "Submit" } 
        }
    }; 

    return modal_content("REGISTER", "register-form", content);  
} 

#[get("/login_form")] 
async fn login_form() {
    let content = html! { 
        form hx-post="/register" hx-target="#" hx-swap="innerHTML" {
            label for="username" { "Username:" } 
            input type="text" name="username" id="username" required; 

            label for="password" { "Password:" } 
            input type="password" name="password" id="password" required; 

            button type="submit" { "Submit" } 
        }
    }; 

    return modal_content("LOGIN", "login-form", content); 
}

#[post("/register", data="<sign_up>")] 
async fn register(sign_up: Json<SignUpRequest>, state: &State<Arc<SessionCache>>) -> Markup {
    let result = register_helper(sign_up.email, sign_up.username, sign_up.password).await;
    let client = PokerClient::localhost(); 
    let cache = state.lock().unwrap(); 
    if let result = Ok(auth_session) { 
        let session = Some(auth_session); 
        cache.insert(session.user_id, session); 
        return html! { p { "User successfully logged in" } }; 
    }
    return html! { p { "Login failed" } }; 
} 

#[post("/login", data="<login>")] 
async fn login(login: Json<LoginRequest>, 
    state: &State<Arc<SessionCache>>) {
    let result = login_helper(login.username, login.password).await; 
    let cache = state.lock().unwrap(); 

    if let result = Ok(auth_session) { 
        let session = Some(auth_session); 
        cache.insert(session.user_id, session); 
    }
} 



#[get("/")]
async fn landing() { 
    layout("", html! { 
        dialog id="register_form" class="rounded p-6 backdrop:bg-black/50" {}
        dialog id="login_form" class="rounded p-6 backdrop:bg-black/50" {}
        form hx-get="/register_form" hx-target="#register-form" hx-swap="innerHTML" hx-on::after-request="document.getElementById('register-form').showModal()" class="pb-3 text-right" { 
            input type="hidden" name="id" 
                value=(id) {}
            button type="submit"
                class="px-4 py-2 font-bold bg-black text-gray-500 border border-gray-300 rounded hover:bg-gray-600 transition-colors" 
                { "REGISTER" } 
        }
        form hx-get="/login_form" hx-target="#login-form" hx-swap="innerHTML" hx-on::after-request="document.getElementById('login-form').showModal()" class="pb-3 text-right" { 
            input type="hidden" name="id" value=(id) {}
            input type="text" name="username" class="px-4 py-2 text-gray-500 border border-gray-300 rounded" 
                placeholder="Enter your username..." { }
            input type="text" name="password" class="px-4 py-2 text-gray-500 border border-gray-300 rounded"
                placeholder="Enter your password..." { }
            button type="submit" class="px-4 py-2 font-boldbg-black text-gray-500 border border-gray-300 rounded hover:bg-gray-600 transition-colors" 
            { "LOGIN" }
        }
    }) 
}
/*
        } else {
            if let Some(ref s) = session {
                println!("\n=== Welcome, {}! ===", s.username);
            }
            
            println!("\n=== Main Menu ===");
            println!("1. List & Join Games");
            println!("2. Create New Game");
            println!("3. Watch a Game");
            println!("4. Add Chips");
            println!("5. Logout");
            println!("6. Exit");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect("Failed to read line");

            match choice.trim() {
                "1" => list_and_join_games(&client, session.as_ref().unwrap()).await,
                "2" => create_new_game(&client, session.as_ref().unwrap()).await,
                "3" => watch_game(&client, session.as_ref().unwrap()).await,
                "4" => {
                    println!("Enter amount of chips to add: ");
                    let amount_str = read_input("Amount: ");
                    let amount = match amount_str.trim().parse::<u32>() {
                        Ok(n) => n,
                        Err(_) => {
                            println!("Invalid amount. Enter a number.");
                            continue;
                        }
                    };
                    match client.add_chips(&session.as_ref().unwrap().user_id, amount).await {
                        Ok(resp) => {
                            if resp.success {
                                println!("{} (chips added: {})", resp.message, resp.chips_added);
                            } else {
                                println!("Error: {}", resp.message);
                            }
                        }
                        Err(e) => println!("Error adding chips: {}", e),
                    }
                    continue;
                }
                "5" => {
                    session = None;
                    authenticated = false;
                    println!("Logged out successfully.");
                }
                "6" => {
                    println!("Exiting the game. Goodbye!");
                    break;
                }
                _ => println!("Invalid choice, please try again."),
            }
        }
    }
}
*/
