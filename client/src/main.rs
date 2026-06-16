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
use client::read_input;

fn banner(text: &str) {
    let width = text.len() + 6;
    println!("{}", "=".repeat(width));
    println!("== {} ==", text);
    println!("{}", "=".repeat(width));
}

async fn list_and_join_games(client: &PokerClient, session: &AuthSession) {
    // List all games
    match client.list_games().await {
        Ok(response) => {
            println!("\n=== Available Games ===");
            if response.games.is_empty() {
                println!("No games available. Create one from the main menu!");
                return;
            }
            
            println!(
                "{:<5} {:<40} {:<18} {:<10} {:<10}",
                "#", "Game ID", "Type", "Players", "Status"
            );
            println!("{}", "-".repeat(83));

            for (idx, game) in response.games.iter().enumerate() {
                let status = match game.status {
                    GameStatus::WaitingForPlayers => "Waiting",
                    GameStatus::InProgress => "In Progress",
                    GameStatus::Finished => "Finished",
                };
                println!(
                    "{:<5} {:<40} {:<18} {}/{:<7} {:<10}",
                    idx + 1,
                    game.game_id,
                    game.game_type,
                    game.player_count,
                    game.max_players,
                    status
                );
            }
            
            println!("\nEnter game number to join (or 0 to go back):");
            let choice = read_input("Choice: ");
            
            match choice.trim().parse::<usize>() {
                Ok(0) => {
                    println!("Returning to main menu...");
                    return;
                }
                Ok(num) if num > 0 && num <= response.games.len() => {
                    let selected_game = &response.games[num - 1];
                    if let Err(e) = game_settings::join_and_play_game(
                        client,
                        session,
                        &selected_game.game_id,
                        selected_game.game_type,
                    ).await {
                        println!("Error joining game: {}", e);
                    }
                }
                _ => {
                    println!("Invalid choice.");
                }
            }
        }
        Err(e) => {
            println!("Error listing games: {}", e);
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

#[tokio::main]
async fn main() {
    banner("Welcome to KrustyRabz Poker");
    let mut authenticated = false;
    let mut session: Option<AuthSession> = None;
    let client = PokerClient::localhost();
    
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
                            session = Some(auth_session);
                            authenticated = true;
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
