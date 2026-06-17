// Common game logic shared across all poker variants
//
// This module contains reusable functions for creating, joining, listing,
// and playing poker games regardless of the variant.

use crate::api::PokerClient;
use crate::api::client::ApiError;
use crate::authentication::AuthSession;
use poker_core::{BettingRound, GameStateUpdate, GameStatus, GameType};
use client::read_input;

/// Create a new game and enter the game loop. Only Five Card Draw is supported.
pub async fn create_and_play_game(
    client: &PokerClient,
    session: &AuthSession,
    game_type: GameType,
) -> Result<(), ApiError> {
    println!("\nCreating new {} game...", game_type);

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

/// Join an existing game and enter the game loop. Only Five Card Draw is fully supported.
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
#[allow(dead_code)]
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

        // Show action menu (only offer actions that are valid this turn/phase)
        let hand_not_started = state.your_hand.is_empty();

        // If it's not Drawing or Showdown, AND the hand has actually started, it's a live betting street
        let is_betting_phase = !matches!(
            state.betting_round,
            BettingRound::Drawing | BettingRound::Showdown
        ) && !hand_not_started;
        
        let is_draw_phase = state.betting_round == BettingRound::Drawing;
        let is_my_turn = state.action_on.as_deref() == Some(session.username.as_str());

        // Any game can start if cards haven't been dealt and there are enough players
        let can_start_hand = hand_not_started && state.player_count >= 2;

        println!("\n=== Actions ===");
        if can_start_hand {
            println!("9. Start hand (deal cards, need 2+ players)");
        }
        println!("1. Refresh Game State");
        if is_betting_phase && is_my_turn {
            println!("2. Fold");
            println!("3. Check");
            println!("4. Call");
            println!("5. Bet");
            println!("6. Raise");
        }
        if has_draw_phase {
            if is_draw_phase && is_my_turn {
                println!("7. Draw Cards (discard and draw new)");
            }
            println!("8. Leave Game");
        } else {
            println!("7. Leave Game");
        }

        let choice = read_input("Choice: ");

        let result = match choice.trim() {
            "9" if can_start_hand => client.start_hand(game_id, &session.user_id).await,
            "1" => {
                println!("Refreshing...");
                continue;
            }
            "2" if is_betting_phase && is_my_turn => client.fold(&session.user_id, game_id).await,
            "3" if is_betting_phase && is_my_turn => client.check(&session.user_id, game_id).await,
            "4" if is_betting_phase && is_my_turn => client.call(&session.user_id, game_id).await,
            "5" if is_betting_phase && is_my_turn => {
                let amount = read_input("Enter bet amount: ");
                match amount.trim().parse::<u32>() {
                    Ok(amt) => client.bet(&session.user_id, game_id, amt).await,
                    Err(_) => {
                        println!("Invalid amount");
                        continue;
                    }
                }
            }
            "6" if is_betting_phase && is_my_turn => {
                let amount = read_input("Enter raise amount: ");
                match amount.trim().parse::<u32>() {
                    Ok(amt) => client.raise(&session.user_id, game_id, amt).await,
                    Err(_) => {
                        println!("Invalid amount");
                        continue;
                    }
                }
            }
            "7" if has_draw_phase && is_draw_phase && is_my_turn => {
                println!("Enter card positions to discard (1-5), separated by spaces (max 3).");
                println!("Example: '1 3 5' discards cards 1, 3, and 5. Enter 0 or leave empty to keep all cards (stand pat).");
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
            "8" if has_draw_phase => {
                println!("Leaving game...");
                break;
            }
            "7" if has_draw_phase => {
                // Draw phase but not your turn, or not in draw phase
                println!("Invalid choice (Draw only when it's your turn in Drawing phase)");
                continue;
            }
            "7" => {
                println!("Leaving game...");
                break;
            }
            _ => {
                println!("Invalid choice (use 1 to refresh; betting/draw only when it's your turn)");
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
    if let Some(ref msg) = state.last_hand_message {
        println!("\n{}", msg);
    }
    println!("\n{}", "=".repeat(60));
    println!("  GAME: {} | POT: ${}", state.game_id, state.pot);
    println!("  Round: {} | Current Bet: ${}", state.betting_round, state.current_bet);
    println!("{}", "=".repeat(60));

    if !state.community_cards.is_empty() {
        println!("\n--- Community Cards ---");
        println!("  {}", state.community_cards.join("  "));
    }

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
