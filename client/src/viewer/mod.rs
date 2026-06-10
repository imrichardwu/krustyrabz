// Viewer module: watch a game without playing.
//
// - Viewer = client that is not playing but can watch game activity and request results.
// - Flow: list games → pick game → register as viewer → poll public game state (read-only).
// - Server returns same GameStateUpdate for viewers but with your_hand empty, your_chips 0.
// - Viewer can refresh to see updates and leave when done.

use crate::api::PokerClient;
use crate::authentication::AuthSession;
use poker_core::{GameStateUpdate, GameStatus};
use client::read_input;

/// Watch a game (read-only). Register as viewer, then poll and display public state.
pub async fn watch_game(client: &PokerClient, session: &AuthSession) {
    println!("\n{}", "=".repeat(50));
    println!("        WATCH A GAME");
    println!("{}", "=".repeat(50));

    // List games
    let list = match client.list_games().await {
        Ok(l) => l,
        Err(e) => {
            println!("Error listing games: {}", e);
            return;
        }
    };

    // Filter to only show games that are waiting or in progress (exclude finished)
    let watchable_games: Vec<_> = list.games.iter()
        .filter(|g| g.status == GameStatus::WaitingForPlayers || g.status == GameStatus::InProgress)
        .collect();

    if watchable_games.is_empty() {
        println!("No games available to watch (all games are finished).");
        return;
    }

    println!("\n--- Available games to watch ---");
    println!(
        "{:<4} {:<40} {:<18} {:<10} {:<10}",
        "#", "Game ID", "Type", "Players", "Status"
    );
    println!("{}", "-".repeat(82));
    for (i, g) in watchable_games.iter().enumerate() {
        let status = match g.status {
            GameStatus::WaitingForPlayers => "Waiting",
            GameStatus::InProgress => "In Progress",
            GameStatus::Finished => "Finished",
        };
        println!(
            "{:<4} {:<40} {:<18} {}/{:<7} {:<10}",
            i + 1,
            g.game_id,
            g.game_type,
            g.player_count,
            g.max_players,
            status
        );
    }

    let choice = read_input("Enter game number (or 0 to cancel): ");
    let idx: usize = match choice.trim().parse::<usize>() {
        Ok(n) if n == 0 => {
            println!("Cancelled.");
            return;
        }
        Ok(n) if n >= 1 && n <= watchable_games.len() => n - 1,
        _ => {
            println!("Invalid choice.");
            return;
        }
    };

    let game_id = watchable_games[idx].game_id.clone();
    println!("\nRegistering as viewer for game {}...", game_id);

    if let Err(e) = client
        .register_viewer(&session.user_id, &game_id)
        .await
    {
        println!("Error registering as viewer: {}", e);
        return;
    }
    println!("Watching game. (You will see public state only—no player hands.)\n");

    viewer_loop(client, &session.user_id, &game_id).await;
}

async fn viewer_loop(
    client: &PokerClient,
    viewer_id: &str,
    game_id: &str,
) {
    loop {
        let state = match client.get_game(game_id, viewer_id).await {
            Ok(s) => s,
            Err(e) => {
                println!("Error fetching game state: {}", e);
                break;
            }
        };

        display_public_state(&state);

        println!("\n--- Viewer actions ---");
        println!("1. Refresh");
        println!("2. Leave");

        let choice = read_input("Choice: ");
        match choice.trim() {
            "1" => continue,
            "2" => {
                println!("Leaving spectator mode.");
                break;
            }
            _ => println!("Invalid choice."),
        }
    }
}

fn display_public_state(state: &GameStateUpdate) {
    println!("\n{}", "=".repeat(60));
    println!("  GAME: {} | POT: ${}", state.game_id, state.pot);
    println!("  Round: {} | Current Bet: ${}", state.betting_round, state.current_bet);
    println!("{}", "=".repeat(60));

    println!("\n--- Players ---");
    for p in &state.players {
        let dealer = if p.is_dealer { " (D)" } else { "" };
        let folded = if p.folded { " [FOLDED]" } else { "" };
        let action = if state.action_on.as_ref() == Some(&p.username) {
            " << ACTION"
        } else {
            ""
        };
        println!(
            "  {} - Chips: ${} | Bet: ${} | Cards: {}{}{}{}",
            p.username, p.chips, p.current_bet, p.cards_count, dealer, folded, action
        );
    }

    if !state.community_cards.is_empty() {
        println!("\n--- Community cards ---");
        for c in &state.community_cards {
            print!("  {} ", c);
        }
        println!();
    }

    if let Some(ref who) = state.action_on {
        println!("\n  >>> {} to act <<<", who);
    }
}
