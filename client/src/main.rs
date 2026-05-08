mod api;
mod games;
mod authentication;
mod viewer;
mod player;

use crate::api::PokerClient;
use games::{five_card_draw, seven_card_stud, texas_holdem};
use authentication::{login, register, AuthSession};

fn banner(text: &str) {
    let width = text.len() + 6;
    println!("{}", "=".repeat(width));
    println!("== {} ==", text);
    println!("{}", "=".repeat(width));
}

#[tokio::main]
async fn main() {
    banner("Welcome to KrustyRabz Poker");
    let mut authenticated = false;
    let mut session: Option<AuthSession> = None;
    
    loop {
        if !authenticated {
            println!("\n=== Authentication ===");
            println!("1. Register");
            println!("2. Login");
            println!("3. Exit");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect("Failed to read line");

            match choice.trim() {
                "1" => {
                    match register().await {
                        Ok(auth_session) => {
                            
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                "2" => {
                    match login().await {
                        Ok(auth_session) => {
                            session = Some(auth_session);
                            authenticated = true;
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                "3" => {
                    println!("Exiting the game. Goodbye!");
                    return;
                }
                _ => println!("Invalid choice, please try again."),
            }
        } else {
            if let Some(ref s) = session {
                println!("\n=== Welcome, {}! ===", s.username);
            }
            
            println!("\n=== Game Selection ===");
            println!("1. Five Card Draw");
            println!("2. Seven Card Stud");
            println!("3. Texas Hold'em");
            println!("4. Logout");
            println!("5. Exit");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect("Failed to read line");

            match choice.trim() {
                "1" => five_card_draw(),
                "2" => seven_card_stud(),
                "3" => texas_holdem(),
                "4" => {
                    session = None;
                    authenticated = false;
                    println!("Logged out successfully.");
                }
                "5" => {
                    println!("Exiting the game. Goodbye!");
                    break;
                }
                _ => println!("Invalid choice, please try again."),
            }
        }
    }
}
