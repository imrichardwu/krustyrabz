// Five Card Draw - Client Game Interface
//
// This module provides the CLI interface for playing Five Card Draw poker.

use crate::api::PokerClient;
use crate::api::client::ApiError;
use crate::authentication::AuthSession;
use poker_core::{BettingRound, GameStateUpdate, GameStatus, GameType};
use client::read_input;

/// Main entry point for Five Card Draw game.
pub async fn five_card_draw(client: &PokerClient, session: &AuthSession) {
    println!("\n{}", "=".repeat(50));
    println!("        FIVE CARD DRAW POKER");
    println!("{}", "=".repeat(50));

    loop {
        println!("\n=== Five Card Draw Menu ===");
        println!("1. Create New Game");
        println!("2. Join Existing Game");
        println!("3. List Available Games");
        println!("4. Back to Main Menu");

        let choice = read_input("Choice: ");

        match choice.trim() {
            "1" => {
                if let Err(e) = create_and_play_game(client, session).await {
                    println!("Error: {}", e);
                }
            }
            "2" => {
                let game_id = read_input("Enter Game ID: ");
                if let Err(e) = join_and_play_game(client, session, game_id.trim()).await {
                    println!("Error: {}", e);
                }
            }
            "3" => {
                if let Err(e) = list_games(client).await {
                    println!("Error: {}", e);
                }
            }
            "4" => {
                println!("Returning to main menu...");
                return;
            }
            _ => println!("Invalid choice, please try again."),
        }
    }
}

/// Create a new game and enter the game loop.
async fn create_and_play_game(client: &PokerClient, session: &AuthSession) -> Result<(), ApiError> {
    println!("\nCreating new Five Card Draw game...");

    let response = client
        .create_game(&session.user_id, &session.username, GameType::FiveCardDraw)
        .await?;

    if !response.success {
        return Err(ApiError::Server(response.message));
    }

    let game_id = response.game_id.ok_or(ApiError::Server("No game ID returned".to_string()))?;
    println!("Game created! Game ID: {}", game_id);
    println!("Waiting for other players to join...");

    game_loop(client, session, &game_id).await
}

/// Join an existing game and enter the game loop.
async fn join_and_play_game(
    client: &PokerClient,
    session: &AuthSession,
    game_id: &str,
) -> Result<(), ApiError> {
    println!("\nJoining game {}...", game_id);

    let response = client
        .join_game(&session.user_id, &session.username, game_id)
        .await?;

    if !response.success {
        return Err(ApiError::Server(response.message));
    }

    println!("Successfully joined game!");

    game_loop(client, session, game_id).await
}

/// List all available games.
async fn list_games(client: &PokerClient) -> Result<(), ApiError> {
    let response = client.list_games().await?;

    println!("\n=== Available Games ===");
    if response.games.is_empty() {
        println!("No games available. Create one!");
    } else {
        println!(
            "{:<40} {:<15} {:<10} {:<10}",
            "Game ID", "Type", "Players", "Status"
        );
        println!("{}", "-".repeat(75));

        for game in &response.games {
            let status = match game.status {
                GameStatus::WaitingForPlayers => "Waiting",
                GameStatus::InProgress => "In Progress",
                GameStatus::Finished => "Finished",
            };
            println!(
                "{:<40} {:<15} {}/{:<7} {:<10}",
                game.game_id, game.game_type, game.player_count, game.max_players, status
            );
        }
    }
    Ok(())
}

/// Main game loop - displays state and handles player actions.
async fn game_loop(
    client: &PokerClient,
    session: &AuthSession,
    game_id: &str,
) -> Result<(), ApiError> {
    loop {
        // Get current game state
        let state = client.get_game(game_id, &session.user_id).await?;

        // Display the game state
        display_game_state(&state, &session.user_id);

        // Check if game is over
        if state.player_count <= 1 && state.betting_round == BettingRound::Showdown {
            println!("\n*** GAME OVER ***");
            break;
        }

        // Show action menu
        println!("\n=== Actions ===");
        println!("1. Refresh Game State");
        println!("2. Fold");
        println!("3. Check");
        println!("4. Call");
        println!("5. Bet");
        println!("6. Raise");
        println!("7. Draw Cards (discard and draw new)");
        println!("8. Leave Game");

        let choice = read_input("Choice: ");

        let result = match choice.trim() {
            "1" => {
                println!("Refreshing...");
                continue;
            }
            "2" => {
                client.fold(&session.user_id, game_id).await
            }
            "3" => {
                client.check(&session.user_id, game_id).await
            }
            "4" => {
                client.call(&session.user_id, game_id).await
            }
            "5" => {
                let amount = read_input("Enter bet amount: ");
                match amount.trim().parse::<u32>() {
                    Ok(amt) => client.bet(&session.user_id, game_id, amt).await,
                    Err(_) => {
                        println!("Invalid amount");
                        continue;
                    }
                }
            }
            "6" => {
                let amount = read_input("Enter raise amount: ");
                match amount.trim().parse::<u32>() {
                    Ok(amt) => client.raise(&session.user_id, game_id, amt).await,
                    Err(_) => {
                        println!("Invalid amount");
                        continue;
                    }
                }
            }
            "7" => {
                println!("Enter card positions to discard (1-5), separated by spaces.");
                println!("Example: '1 3 5' discards cards 1, 3, and 5");
                let input = read_input("Discard: ");
                let indices: Vec<usize> = input
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .filter(|&n| n >= 1 && n <= 5)
                    .map(|n| n - 1) // Convert to 0-indexed
                    .collect();

                if indices.len() > 3 {
                    println!("You can only discard up to 3 cards!");
                    continue;
                }

                client.draw(&session.user_id, game_id, indices).await
            }
            "8" => {
                println!("Leaving game...");
                break;
            }
            _ => {
                println!("Invalid choice");
                continue;
            }
        };

        // Handle action result
        match result {
            Ok(response) => {
                if response.success {
                    println!("Action successful: {}", response.message);
                } else {
                    println!("Action failed: {}", response.message);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Display the current game state.
fn display_game_state(state: &GameStateUpdate, _my_player_id: &str) {
    println!("\n{}", "=".repeat(60));
    println!("  GAME: {} | POT: ${}", state.game_id, state.pot);
    println!("  Round: {} | Current Bet: ${}", state.betting_round, state.current_bet);
    println!("{}", "=".repeat(60));

    // Display players
    println!("\n--- Players ---");
    for player in &state.players {
        let dealer_mark = if player.is_dealer { " (D)" } else { "" };
        let folded_mark = if player.folded { " [FOLDED]" } else { "" };
        let action_mark = if state.action_on.as_ref() == Some(&player.username) {
            " << ACTION"
        } else {
            ""
        };

        println!(
            "  {} - Chips: ${} | Bet: ${} | Cards: {}{}{}{}",
            player.username,
            player.chips,
            player.current_bet,
            player.cards_count,
            dealer_mark,
            folded_mark,
            action_mark
        );
    }

    // Display your hand
    if !state.your_hand.is_empty() {
        println!("\n--- Your Hand ---");
        print!("  ");
        for (i, card) in state.your_hand.iter().enumerate() {
            print!("[{}] {} ", i + 1, card);
        }
        println!();
    }

    // Display your chips
    println!("\n  Your Chips: ${}", state.your_chips);

    // Show whose turn it is
    if let Some(ref action_player) = state.action_on {
        println!("\n  >>> Waiting for {} to act <<<", action_player);
    }
}