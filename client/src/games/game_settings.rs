// Common game logic shared across all poker variants
//
// This module contains reusable functions for creating, joining, listing,
// and playing poker games regardless of the variant.

use crate::api::PokerClient;
use crate::api::client::ApiError;
use crate::authentication::AuthSession;
use poker_core::{BettingRound, GameStateUpdate, GameStatus, GameType};
use client::read_input;

/// Create a new game and enter the game loop.
pub async fn create_and_play_game(
    client: &PokerClient,
    session: &AuthSession,
    game_type: GameType,
) -> Result<(), ApiError> {
    let game_name = match game_type {
        GameType::FiveCardDraw => "Five Card Draw",
        GameType::SevenCardStud => "Seven Card Stud",
        GameType::TexasHoldEm => "Texas Hold'em",
    };
    
    println!("\nCreating new {} game...", game_name);

    let response = client
        .create_game(&session.user_id, &session.username, game_type)
        .await?;

    if !response.success {
        return Err(ApiError::Server(response.message));
    }

    let game_id = response.game_id.ok_or(ApiError::Server("No game ID returned".to_string()))?;
    println!("Game created! Game ID: {}", game_id);
    println!("Waiting for other players to join...");

    game_loop(client, session, &game_id, game_type).await
}

/// Join an existing game and enter the game loop.
pub async fn join_and_play_game(
    client: &PokerClient,
    session: &AuthSession,
    game_id: &str,
    game_type: GameType,
) -> Result<(), ApiError> {
    println!("\nJoining game {}...", game_id);

    let response = client
        .join_game(&session.user_id, &session.username, game_id)
        .await?;

    if !response.success {
        return Err(ApiError::Server(response.message));
    }

    println!("Successfully joined game!");

    game_loop(client, session, game_id, game_type).await
}

/// List all available games.
pub async fn list_games(client: &PokerClient) -> Result<(), ApiError> {
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
pub async fn game_loop(
    client: &PokerClient,
    session: &AuthSession,
    game_id: &str,
    game_type: GameType,
) -> Result<(), ApiError> {
    let has_draw_phase = matches!(game_type, GameType::FiveCardDraw);
    
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
        
        if has_draw_phase {
            println!("7. Draw Cards (discard and draw new)");
            println!("8. Leave Game");
        } else {
            println!("7. Leave Game");
        }

        let choice = read_input("Choice: ");

        let result = match choice.trim() {
            "1" => {
                println!("Refreshing...");
                continue;
            }
            "2" => client.fold(&session.user_id, game_id).await,
            "3" => client.check(&session.user_id, game_id).await,
            "4" => client.call(&session.user_id, game_id).await,
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
                if has_draw_phase {
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
                } else {
                    println!("Leaving game...");
                    break;
                }
            }
            "8" => {
                if has_draw_phase {
                    println!("Leaving game...");
                    break;
                } else {
                    println!("Invalid choice");
                    continue;
                }
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
pub fn display_game_state(state: &GameStateUpdate, _my_player_id: &str) {
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
