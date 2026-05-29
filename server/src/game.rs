use crate::betting::{BettingRound, BettingState};
use crate::deck::Deck;
use crate::player::Player;
use crate::table::Table;
use poker_core::hand::{Hand, HandRank};
use poker_core::{Card, CardType, Rank, Suit};
use strum::IntoEnumIterator;
use strum_macros::Display;
use uuid::Uuid;

/// Converts a deck index (0..52) to a Card using the same order as Deck::construct():
/// suit then rank (Clubs 2..A, Diamonds 2..A, Hearts 2..A, Spades 2..A).
fn deck_index_to_card(i: u8) -> Option<Card> {
    if i >= 52 {
        return None;
    }
    let suit_idx = (i / 13) as usize;
    let rank_idx = (i % 13) as usize;
    let suit = Suit::iter().nth(suit_idx)?;
    let rank = Rank::iter().nth(rank_idx)?;
    Some(Card::construct(rank, suit, CardType::Private))
}

#[derive(Debug, Clone)]
pub struct SharedGameState {
    pub deck: Deck,
    pub table: Table,
    pub pot: u32,

    // Core Game
    pub dealer_idx: usize,
    pub action_on: Option<Uuid>,  

    // Shared Betting Data
    pub betting_state: BettingState,
}

impl SharedGameState {
    pub fn new(capacity: usize) -> Self {
        Self {
            deck: Deck::standard(),
            table: Table::with_max_players(capacity),
            pot: 0,
            dealer_idx: 0,
            action_on: None,
            betting_state: BettingState::new(),
        }
    }

    pub fn get_player_index(&self, player_id: Uuid) -> Option<usize> {
        self.table.players.iter().position(|p| p.id == player_id)
    }

    pub fn get_active_player_mut(&mut self) -> Option<&mut Player> {
        match self.action_on {
            Some(uid) => self.table.get_player_mut(uid),
            None => None,
        }
    }

    pub fn count_active_players(&self) -> usize {
        self.table.players.iter().filter(|p| !p.is_folded).count()
    }

    fn next_player_index(&self, uid: Uuid) -> usize {
        let i = self
            .get_player_index(uid)
            .expect("action_on refers to non-existent player");

        return i + 1;
    }

    /// Rotates the action to the next eligible player.
    /// Usage Notes -
    /// include_all_in (bool flag) - If true, players with 0 chips (All-In) get a turn
    ///   Use false for Betting Rounds
    ///   Use true for Draw/Showdown Rounds
    pub fn advance_action(&mut self, include_all_in: bool) -> bool {
        if self.table.players.is_empty() {
            return false;
        }

        let start_index = match self.action_on {
            Some(uid) => self.next_player_index(uid),
            None => self.dealer_idx + 1,
        };

        let count = self.table.players.len();

        for i in 0..count {
            let idx = (start_index + i) % count;
            let player = &self.table.players[idx];

            if player.is_folded {
                continue;
            }

            if include_all_in || player.chips > 0 {
                self.action_on = Some(player.id);
                return true;
            }
        }

        self.action_on = None;
        return false;
    }

    pub fn post_bet(&mut self, player_id: Uuid, amount: u32) -> Result<(), &'static str> {
        // Lookup player
        let player = self
            .table
            .get_player_mut(player_id)
            .ok_or("player_not_found")?;

        if amount > player.chips {
            return Err("insufficient_funds");
        }

        player.chips -= amount; // Wallet decreases
        player.current_bet += amount; // Ledger updates
        self.pot += amount; // Pot grows

        Ok(())
    }

    pub fn reset_current_bets(&mut self) {
        for player in &mut self.table.players {
            player.current_bet = 0;
        }
        self.betting_state.reset_round();
    }

    /// Generic handler for standard betting rounds (PreDraw, PostDraw) [Potentially useful later for other variants too]
    /// Returns Ok(true) if the round is over
    /// Returns Ok(false) if the round continues
    pub fn process_betting_action(
        &mut self,
        player_id: Uuid,
        action: poker_core::GameAction,
    ) -> Result<bool, String> {
        if self.action_on != Some(player_id) {
            return Err("not_your_turn".to_string());
        }

        let player_idx = self.get_player_index(player_id).unwrap();
        let current_contribution = self.table.players[player_idx].current_bet;
        let to_call = self.betting_state.to_call;

        match action {
            poker_core::GameAction::Fold => {
                let player = self.table.get_player_mut(player_id).unwrap();
                player.is_folded = true;
            }

            poker_core::GameAction::Check => {
                if current_contribution < to_call {
                    return Err("cannot_check_must_call".to_string());
                }
            }

            poker_core::GameAction::Call => {
                let needed = to_call.saturating_sub(current_contribution);
                self.post_bet(player_id, needed)?;
            }

            poker_core::GameAction::Bet { amount } => {
                if to_call > 0 {
                    return Err("cannot_bet_must_raise".to_string());
                }
                if amount < self.betting_state.min_raise {
                    return Err("bet_too_small".to_string());
                }

                // Bet 10 means Raise To 10 (from 0)
                self.post_bet(player_id, amount)?;

                self.betting_state.to_call = amount;
                self.betting_state.last_aggressor = Some(player_id);
            }

            poker_core::GameAction::Raise { amount } => {
                if to_call == 0 {
                    return Err("cannot_raise_must_bet".to_string());
                }
                if amount < to_call + self.betting_state.min_raise {
                    return Err("raise_too_small".to_string());
                }

                // "Raise To 50", in for 10, pay 40 more
                let delta = amount.saturating_sub(current_contribution);
                self.post_bet(player_id, delta)?;

                self.betting_state.to_call = amount;
                self.betting_state.raises_used += 1;
                self.betting_state.last_aggressor = Some(player_id);
            }

            _ => return Err("invalid_action_for_betting_phase".to_string()),
        }

        let was_aggressor = self.betting_state.last_aggressor;

        // skip All-Ins
        if !self.advance_action(false) {
            return Ok(true);
        }

        if let Some(aggressor) = was_aggressor {
            // Looped back to the raiser -> Round Over
            if self.action_on == Some(aggressor) {
                return Ok(true);
            }
        } else {
            // Check-around complete
            // Logic: back at Dealer+1 and pot is flat
            let start_idx = (self.dealer_idx + 1) % self.table.players.len();
            let current_idx = self.get_player_index(self.action_on.unwrap()).unwrap();

            if current_idx == start_idx && self.betting_state.to_call == 0 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Evaluates all hands, splits the pot, and pays the winner(s).
    /// Returns a list of (PlayerId, AmountWon) for notification.
    ///
    /// Player hands are stored as Vec<u8> (deck indices 0..52). Indices use the same
    /// order as Deck::construct(): suit then rank (Clubs 2..A, Diamonds 2..A, etc.).
    /// Supports 5-card hands (e.g. Five Card Draw) and 7-card hands (e.g. Stud); for 7
    /// cards the best 5-card hand is evaluated.
    pub fn resolve_showdown(&mut self) -> Vec<(Uuid, u32)> {
        let active_players: Vec<(Uuid, Vec<u8>)> = self
            .table
            .players
            .iter()
            .filter(|p| !p.is_folded)
            .map(|p| (p.id, p.hand.clone()))
            .collect();

        if active_players.is_empty() {
            return Vec::new();
        }

        let pot = self.pot;
        if pot == 0 {
            return Vec::new();
        }

        // Build (player_id, HandRank) for each player with a valid evaluable hand
        let mut evaluated: Vec<(Uuid, HandRank)> = Vec::with_capacity(active_players.len());
        for (player_id, indices) in &active_players {
            let cards: Vec<Card> = indices
                .iter()
                .filter_map(|&i| deck_index_to_card(i))
                .collect();

            let hand_rank = match cards.len() {
                5 => {
                    let mut hand = Hand::new();
                    for c in &cards {
                        hand.add(*c);
                    }
                    hand.evaluate()
                }
                7 => {
                    let mut hand = Hand::new();
                    for c in &cards {
                        hand.add(*c);
                    }
                    hand.evaluate()
                }
                _ => continue, // Skip players without 5 or 7 cards
            };
            evaluated.push((*player_id, hand_rank));
        }

        if evaluated.is_empty() {
            return Vec::new();
        }

        // Find best hand rank
        let best_rank = evaluated
            .iter()
            .map(|(_, r)| r)
            .max()
            .expect("at least one evaluated hand");

        // All players with the best rank win (split pot)
        let winners: Vec<Uuid> = evaluated
            .iter()
            .filter(|(_, r)| r == best_rank)
            .map(|(id, _)| *id)
            .collect();

        let num_winners = winners.len() as u32;
        let base_payout = pot / num_winners;
        let remainder = pot % num_winners;

        for (i, &winner_id) in winners.iter().enumerate() {
            let payout = base_payout + if (i as u32) < remainder { 1 } else { 0 };
            if let Some(player) = self.table.get_player_mut(winner_id) {
                player.chips += payout;
            }
        }

        self.pot = 0;

        winners
            .iter()
            .enumerate()
            .map(|(i, &id)| {
                let payout = base_payout + if (i as u32) < remainder { 1 } else { 0 };
                (id, payout)
            })
            .collect()
    }

    /// Cleans up state to prepare for the next hand
    pub fn reset_for_new_hand(&mut self) {
        // Reset Deck
        self.deck = Deck::standard();
        self.deck.shuffle();

        // Rotate Dealer
        let count = self.table.players.len();
        if count > 0 {
            self.dealer_idx = (self.dealer_idx + 1) % count;
        }

        // Reset Players
        for player in &mut self.table.players {
            player.hand.clear();
            player.is_folded = false;
            player.current_bet = 0;
        }

        // Reset State
        self.betting_state.reset_round();
        self.action_on = None;
    }
}

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

impl Game {
    /// Returns the game_id as a Uuid
    pub fn get_game_id(&self) -> Uuid {
        match self {
            Game::FiveCardDraw(game) => game.game_id,
            Game::SevenCardStud(game) => game.game_id,
            Game::TexasHoldEm(game) => game.game_id,
        }
    }

    /// Returns the game type
    pub fn get_game_type(&self) -> poker_core::GameType {
        match self {
            Game::FiveCardDraw(_) => poker_core::GameType::FiveCardDraw,
            Game::SevenCardStud(_) => poker_core::GameType::SevenCardStud,
            Game::TexasHoldEm(_) => poker_core::GameType::TexasHoldEm,
        }
    }

    /// Returns the current pot size
    pub fn get_pot(&self) -> u32 {
        match self {
            Game::FiveCardDraw(game) => game.core.pot,
            Game::SevenCardStud(game) => game.core.pot,
            Game::TexasHoldEm(game) => game.core.pot,
        }
    }

    /// Returns the number of players currently in the game
    pub fn get_player_count(&self) -> usize {
        match self {
            Game::FiveCardDraw(game) => game.core.table.players.len(),
            Game::SevenCardStud(game) => game.core.table.players.len(),
            Game::TexasHoldEm(game) => game.core.table.players.len(),
        }
    }

    /// Returns the maximum number of players allowed
    pub fn get_max_players(&self) -> usize {
        match self {
            Game::FiveCardDraw(game) => game.core.table.max_players,
            Game::SevenCardStud(game) => game.core.table.max_players,
            Game::TexasHoldEm(game) => game.core.table.max_players,
        }
    }

    /// Returns true if the table is full
    pub fn is_full(&self) -> bool {
        self.get_player_count() >= self.get_max_players()
    }

    /// Returns the current game status
    pub fn get_status(&self) -> poker_core::GameStatus {
        match self {
            Game::FiveCardDraw(game) => {
                if game.core.table.players.len() < 2 {
                    poker_core::GameStatus::WaitingForPlayers
                } else if game.betting_round == BettingRound::PreDeal {
                    poker_core::GameStatus::WaitingForPlayers
                } else {
                    poker_core::GameStatus::InProgress
                }
            }
            Game::SevenCardStud(game) => {
                if game.core.table.players.len() < 2 {
                    poker_core::GameStatus::WaitingForPlayers
                } else {
                    poker_core::GameStatus::InProgress
                }
            }
            Game::TexasHoldEm(game) => {
                if game.core.table.players.len() < 2 {
                    poker_core::GameStatus::WaitingForPlayers
                } else {
                    poker_core::GameStatus::InProgress
                }
            }
        }
    }

    /// Adds a player to the game
    pub fn add_player(&mut self, player: crate::player::Player) -> Result<(), String> {
        match self {
            Game::FiveCardDraw(game) => {
                game.core.table.seat_player(player)
                    .map_err(|e| e.to_string())
            }
            Game::SevenCardStud(game) => {
                game.core.table.seat_player(player)
                    .map_err(|e| e.to_string())
            }
            Game::TexasHoldEm(game) => {
                game.core.table.seat_player(player)
                    .map_err(|e| e.to_string())
            }
        }
    }

    /// Removes a player from the game
    pub fn remove_player(&mut self, player_id: Uuid) -> Result<(), String> {
        match self {
            Game::FiveCardDraw(game) => {
                game.core.table.remove_player_from_table(player_id)
                    .map_err(|e| e.to_string())
            }
            Game::SevenCardStud(game) => {
                game.core.table.remove_player_from_table(player_id)
                    .map_err(|e| e.to_string())
            }
            Game::TexasHoldEm(game) => {
                game.core.table.remove_player_from_table(player_id)
                    .map_err(|e| e.to_string())
            }
        }
    }
}

// ============================================================================
// Five Card Draw
// ============================================================================

#[derive(Debug, Clone)]
pub struct FiveCardDraw {
    pub game_id: Uuid,
    pub core: SharedGameState,
    pub betting_round: BettingRound,
}

impl FiveCardDraw {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(5),
            betting_round: BettingRound::PreDeal,
        }
    }

    pub fn predraw_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::GameAction,
    ) -> Result<(), String> {
        if self.betting_round != BettingRound::PreDraw {
            return Err("wrong_phase".to_string());
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        if round_over {
            self.transition_to_draw_phase();
        }

        Ok(())
    }

    /// Same logic as predraw, just different phase check and transition
    pub fn postdraw_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::GameAction,
    ) -> Result<(), String> {
        if self.betting_round != BettingRound::PostDraw {
            return Err("wrong_phase".to_string());
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        if round_over {
            self.transition_to_showdown();
        }

        Ok(())
    }

    fn transition_to_draw_phase(&mut self) {
        self.core.reset_current_bets();
        self.betting_round = BettingRound::Drawing;
        self.core.action_on = None;

        // pass true to include All-In players for card swap
        if !self.core.advance_action(true) {
            self.transition_to_post_draw();
        }
    }

    /// called when the last player has finished drawing.
    fn transition_to_post_draw(&mut self) {
        self.core.reset_current_bets();
        self.betting_round = BettingRound::PostDraw;
        self.core.action_on = None;

        // pass false to SKIP All-In players
        if !self.core.advance_action(false) {
            self.transition_to_showdown();
        }
    }

    fn transition_to_showdown(&mut self) {
        self.core.reset_current_bets();
        // self.betting_round = BettingRound::Showdown; 
        //(TODO I am assuming we dont need to actually pause the game for it now in CLI, 
        //will need to account for this in an actual UI)

        let _results = self.core.resolve_showdown();

        self.core.reset_for_new_hand();

        // loop back to PreDeal waiting for next start
        self.betting_round = BettingRound::PreDeal;
    }

    /// Handles a player's draw action (discarding and drawing new cards).
    ///
    /// TODO: Implement proper draw logic once server Player type
    /// uses the Hand type from poker_core with draw() method
    pub fn handle_draw_action(
        &mut self,
        player_id: Uuid,
        discard_indices: Vec<usize>,
    ) -> Result<(), String> {
        if self.core.action_on != Some(player_id) {
            return Err("not_your_turn".to_string());
        }
        if self.betting_round != BettingRound::Drawing {
            return Err("wrong_phase_expecting_drawing".to_string());
        }

        if discard_indices.len() > 5 {
            return Err("too_many_discards".to_string());
        }

        // TODO: Implement card drawing
        // Current server Player has hand: Vec<u8>, not Hand type with draw()
        // For now, just advance the action without actually drawing cards
        let _player = self.core.table.get_player_mut(player_id).unwrap();
        // player.draw(&mut self.core.deck, &discard_indices)?;

        // pass true because the next person might be All-In
        if !self.core.advance_action(true) {
            self.transition_to_post_draw();
        }

        Ok(())
    }
}

// ============================================================================
// Seven Card Stud
// ============================================================================

#[derive(Debug, Clone)]
pub struct SevenCardStud {
    pub game_id: Uuid,
    pub core: SharedGameState,
    pub betting_round: BettingRound,
}

impl SevenCardStud {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(7), // Seven Card Stud typically allows more players
            betting_round: BettingRound::PreDeal,
        }
    }

    // TODO: Implement Seven Card Stud game logic
    // - Deal initial cards (2 down, 1 up)
    // - Betting rounds for each street (3rd, 4th, 5th, 6th, River)
    // - Showdown
}

// ============================================================================
// Texas Hold'em
// ============================================================================

#[derive(Debug, Clone)]
pub struct TexasHoldEm {
    pub game_id: Uuid,
    pub core: SharedGameState,
    pub betting_round: BettingRound,
    pub community_cards: Vec<poker_core::Card>,
}

impl TexasHoldEm {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(9), // Texas Hold'em can have up to 9-10 players
            betting_round: BettingRound::PreDeal,
            community_cards: Vec::new(),
        }
    }

    // TODO: Implement Texas Hold'em game logic
    // - Deal hole cards (2 cards per player)
    // - PreFlop betting
    // - Deal Flop (3 community cards)
    // - Flop betting
    // - Deal Turn (1 community card)
    // - Turn betting
    // - Deal River (1 community card)
    // - River betting
    // - Showdown
}
