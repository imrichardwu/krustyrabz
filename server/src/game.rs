//  Poker Game Variants
//
// This module contains the game variant structures for the poker server.

use crate::betting::{BettingRound, BettingState};
use crate::deck::Deck;
use crate::table::Table;
use crate::player::Player;
use strum_macros::Display;
use uuid::Uuid;

// ============================================================================
// Game Enum
// ============================================================================

/// This enum is a "generic variant" for populating the House's "live_games" vector,
/// which is a heterogeneous data structure containing live or pending games of poker.
/// Each game can be one of three different variants: FiveCardDraw, SevenCardStud,
/// or TexasHoldEm.
#[derive(Debug, Clone, Display)]
pub enum Game {
    #[strum(serialize = "5CD")]
    FiveCardDraw(FiveCardDraw),
    #[strum(serialize = "7CS")]
    SevenCardStud(SevenCardStud),
    #[strum(serialize = "THE")]
    TexasHoldEm(TexasHoldEm),
}

// ============================================================================
// Five Card Draw
// ============================================================================

#[derive(Debug, Clone)]
pub struct FiveCardDraw {
    pub game_id: Uuid,
    pub deck: Deck,
    pub table: Table,
    pub pot: u32,
    pub betting_round: BettingRound,
    pub betting_state: BettingState,
    pub dealer_position: usize,
    pub action_on: Option<Uuid>, // active player ID
}

impl FiveCardDraw {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            deck: Deck::standard(),
            table: Table::new(),
            pot: 0,
            betting_round: BettingRound::PreDeal,
            betting_state: BettingState::new(),
            dealer_position: 0,
            action_on: None,
        }
    }

    /// Round - Step 1: Shuffle the deck
    pub fn shuffle(&mut self) {
        self.deck = Deck::standard();
        self.deck.shuffle();
    }

    /// Round - Step 2: Each player is dealt five private cards which form their hand.
    pub fn deal(&mut self) -> Result<(), String> {
        // Shuffle first
        self.shuffle();
        
        // Reset game state
        self.pot = 0;
        self.betting_round = BettingRound::PreDraw;
        self.betting_state = BettingState::new();

        // Clear all hands
        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }

        // Deal 5 cards to each player
        for deal in 1..=5 {
            for player in &mut self.table.players {
                // On the very first deal, every player's hand should be empty
                if !player.hand.is_empty() && deal == 1 {
                    return Err(format!(
                        "No cards have been dealt, but {} somehow has {} card(s) in hand. This should be impossible.",
                        player.id,
                        player.hand.len()
                    ));
                }
                let cards = self.deck.deal(1);
                if let Some(_card) = cards.first() {
                    // Store card index (simplified representation)
                    player.hand.push(player.hand.len() as u8);
                }
            }
        }

        // Set action on player after dealer
        if !self.table.players.is_empty() {
            let action_idx = (self.dealer_position + 1) % self.table.players.len();
            self.action_on = Some(self.table.players[action_idx].id);
        }

        Ok(())
    }

    /// Round - Step 3: Pre-draw betting round.
    /// This is the first betting round. It begins with the player to the dealer's left,
    /// which for simplicity is defined as the Player located at index 0 in the Table's
    /// Player Vec.
    pub fn predraw_betting(&mut self) -> Result<(), String> {
        self.betting_round = BettingRound::PreDraw;
        // Betting logic would go here
        // For now, just advance to draw phase
        Ok(())
    }

    /// Round - Step 4: Draw phase where players can discard and draw new cards.
    pub fn draw(&mut self, player_id: Uuid, discard_indices: &[usize]) -> Result<(), String> {
        let player = self.table.players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or("Player not found")?;

        if discard_indices.len() > 3 {
            return Err("Can only discard up to 3 cards".to_string());
        }

        // Remove discarded cards (in reverse order to maintain indices)
        let mut sorted_indices = discard_indices.to_vec();
        sorted_indices.sort_by(|a, b| b.cmp(a));
        
        for &idx in &sorted_indices {
            if idx >= player.hand.len() {
                return Err("Invalid card index".to_string());
            }
            player.hand.remove(idx);
        }

        // Deal new cards
        for _ in 0..discard_indices.len() {
            let cards = self.deck.deal(1);
            if cards.first().is_some() {
                player.hand.push(player.hand.len() as u8);
            }
        }

        Ok(())
    }

    /// Round - Step 5: Post-draw betting round.
    pub fn postdraw_betting(&mut self) -> Result<(), String> {
        self.betting_round = BettingRound::PostDraw;
        // Betting logic would go here
        Ok(())
    }

    /// Round - Step 6: Showdown - compare hands and determine winner.
    pub fn showdown(&self) -> Option<Uuid> {
        // Find active (non-folded) players
        let active_players: Vec<&Player> = self.table.players
            .iter()
            .filter(|p| !p.is_folded)
            .collect();

        if active_players.len() == 1 {
            return Some(active_players[0].id);
        }

        // TODO: Implement hand comparison logic
        // For now, return the first active player
        active_players.first().map(|p| p.id)
    }

    /// Round - Step 7: Payout - distribute pot to winner(s).
    pub fn payout(&mut self, winner_id: Uuid) -> Result<u32, String> {
        let pot = self.pot;
        self.pot = 0;

        let winner = self.table.players
            .iter_mut()
            .find(|p| p.id == winner_id)
            .ok_or("Winner not found")?;

        winner.chips += pot;
        Ok(pot)
    }

    /// Reset the game for a new hand.
    pub fn reset(&mut self) {
        self.pot = 0;
        self.betting_round = BettingRound::PreDeal;
        self.betting_state = BettingState::new();
        self.action_on = None;

        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }

        // Rotate dealer
        if !self.table.players.is_empty() {
            self.dealer_position = (self.dealer_position + 1) % self.table.players.len();
        }
    }

    /// Get the username of the player whose turn it is.
    pub fn get_action_on_username(&self) -> String {
        if let Some(player_id) = self.action_on {
            self.table.players
                .iter()
                .find(|p| p.id == player_id)
                .map(|p| p.username.clone())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl Default for FiveCardDraw {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Seven Card Stud
// ============================================================================

#[derive(Debug, Clone)]
pub struct SevenCardStud {
    pub game_id: Uuid,
    pub deck: Deck,
    pub table: Table,
    pub pot: u32,
    pub betting_round: BettingRound,
    pub betting_state: BettingState,
    pub dealer_position: usize,
    pub action_on: Option<Uuid>,
}

impl SevenCardStud {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            deck: Deck::standard(),
            table: Table::new(),
            pot: 0,
            betting_round: BettingRound::PreDeal,
            betting_state: BettingState::new(),
            dealer_position: 0,
            action_on: None,
        }
    }

    pub fn shuffle(&mut self) {
        self.deck = Deck::standard();
        self.deck.shuffle();
    }

    /// Deal initial cards: 2 down, 1 up to each player.
    pub fn deal(&mut self) -> Result<(), String> {
        self.shuffle();
        self.pot = 0;
        self.betting_round = BettingRound::ThirdStreet;
        self.betting_state = BettingState::new();

        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }

        // Deal 3 cards to each player (2 down, 1 up)
        for _ in 0..3 {
            for player in &mut self.table.players {
                let cards = self.deck.deal(1);
                if cards.first().is_some() {
                    player.hand.push(player.hand.len() as u8);
                }
            }
        }

        // Action starts with player showing lowest card (simplified: first player)
        if !self.table.players.is_empty() {
            self.action_on = Some(self.table.players[0].id);
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.pot = 0;
        self.betting_round = BettingRound::PreDeal;
        self.betting_state = BettingState::new();
        self.action_on = None;

        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }
    }

    pub fn get_action_on_username(&self) -> String {
        if let Some(player_id) = self.action_on {
            self.table.players
                .iter()
                .find(|p| p.id == player_id)
                .map(|p| p.username.clone())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl Default for SevenCardStud {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Texas Hold'Em
// ============================================================================

#[derive(Debug, Clone)]
pub struct TexasHoldEm {
    pub game_id: Uuid,
    pub deck: Deck,
    pub table: Table,
    pub pot: u32,
    pub betting_round: BettingRound,
    pub betting_state: BettingState,
    pub dealer_position: usize,
    pub action_on: Option<Uuid>,
    pub community_cards: Vec<u8>, // The 5 community cards
}

impl TexasHoldEm {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            deck: Deck::standard(),
            table: Table::new(),
            pot: 0,
            betting_round: BettingRound::PreDeal,
            betting_state: BettingState::new(),
            dealer_position: 0,
            action_on: None,
            community_cards: Vec::new(),
        }
    }

    pub fn shuffle(&mut self) {
        self.deck = Deck::standard();
        self.deck.shuffle();
    }

    /// Deal 2 hole cards to each player.
    pub fn deal(&mut self) -> Result<(), String> {
        self.shuffle();
        self.pot = 0;
        self.betting_round = BettingRound::PreFlop;
        self.betting_state = BettingState::new();
        self.community_cards.clear();

        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }

        // Deal 2 hole cards to each player
        for _ in 0..2 {
            for player in &mut self.table.players {
                let cards = self.deck.deal(1);
                if cards.first().is_some() {
                    player.hand.push(player.hand.len() as u8);
                }
            }
        }

        // Action starts with player after big blind (dealer + 3)
        if !self.table.players.is_empty() {
            let action_idx = (self.dealer_position + 3) % self.table.players.len();
            self.action_on = Some(self.table.players[action_idx].id);
        }

        Ok(())
    }

    /// Deal the flop (3 community cards).
    pub fn deal_flop(&mut self) -> Result<(), String> {
        if self.betting_round != BettingRound::PreFlop {
            return Err("Not in pre-flop phase".to_string());
        }

        // Burn one card, deal 3
        self.deck.deal(1); // burn
        for _ in 0..3 {
            let cards = self.deck.deal(1);
            if cards.first().is_some() {
                self.community_cards.push(self.community_cards.len() as u8);
            }
        }

        self.betting_round = BettingRound::Flop;
        Ok(())
    }

    /// Deal the turn (4th community card).
    pub fn deal_turn(&mut self) -> Result<(), String> {
        if self.betting_round != BettingRound::Flop {
            return Err("Not in flop phase".to_string());
        }

        // Burn one, deal one
        self.deck.deal(1);
        let cards = self.deck.deal(1);
        if cards.first().is_some() {
            self.community_cards.push(self.community_cards.len() as u8);
        }

        self.betting_round = BettingRound::Turn;
        Ok(())
    }

    /// Deal the river (5th community card).
    pub fn deal_river(&mut self) -> Result<(), String> {
        if self.betting_round != BettingRound::Turn {
            return Err("Not in turn phase".to_string());
        }

        // Burn one, deal one
        self.deck.deal(1);
        let cards = self.deck.deal(1);
        if cards.first().is_some() {
            self.community_cards.push(self.community_cards.len() as u8);
        }

        self.betting_round = BettingRound::River;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.pot = 0;
        self.betting_round = BettingRound::PreDeal;
        self.betting_state = BettingState::new();
        self.action_on = None;
        self.community_cards.clear();

        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }

        // Rotate dealer
        if !self.table.players.is_empty() {
            self.dealer_position = (self.dealer_position + 1) % self.table.players.len();
        }
    }

    pub fn get_action_on_username(&self) -> String {
        if let Some(player_id) = self.action_on {
            self.table.players
                .iter()
                .find(|p| p.id == player_id)
                .map(|p| p.username.clone())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl Default for TexasHoldEm {
    fn default() -> Self {
        Self::new()
    }
}
