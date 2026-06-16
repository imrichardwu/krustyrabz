use crate::betting::{BettingRound, BettingState};
use crate::deck::Deck;
use crate::player::Player;
use crate::table::Table;
use poker_core::{DeckTrait, GameType, GameStatus};
use strum_macros::Display;
use uuid::Uuid;

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
        action: poker_core::protocol::GameAction,
    ) -> Result<bool, String> {
        if self.action_on != Some(player_id) {
            return Err("not_your_turn".to_string());
        }

        let player_idx = self.get_player_index(player_id).unwrap();
        let current_contribution = self.table.players[player_idx].current_bet;
        let to_call = self.betting_state.to_call;

        match action {
            poker_core::protocol::GameAction::Fold => {
                let player = self.table.get_player_mut(player_id).unwrap();
                player.is_folded = true;
            }

            poker_core::protocol::GameAction::Check => {
                if current_contribution < to_call {
                    return Err("cannot_check_must_call".to_string());
                }
            }

            poker_core::protocol::GameAction::Call => {
                let needed = to_call.saturating_sub(current_contribution);
                self.post_bet(player_id, needed)?;
            }

            poker_core::protocol::GameAction::Bet { amount } => {
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

            poker_core::protocol::GameAction::Raise { amount } => {
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

        // Only one non-folded player left: round over (e.g. everyone else folded)
        if self.count_active_players() == 1 {
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
    pub fn resolve_showdown(&mut self) -> Vec<(Uuid, u32)> {
        let mut candidates: Vec<(Uuid, poker_core::hand::HandRank)> = self
            .table
            .players
            .iter()
            .filter(|p| !p.is_folded)
            .map(|p| (p.id, p.hand.evaluate())) // evaluate() from hand.rs
            .collect();

        if candidates.is_empty() {
            return Vec::new();
        }

        // sort descending (Best hand first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let best_rank = &candidates[0].1;

        // identify winners
        let winners: Vec<Uuid> = candidates
            .iter()
            .take_while(|(_, rank)| rank == best_rank)
            .map(|(id, _)| *id)
            .collect();

        // Split Pot TODO MIGHT NEED REWORKING THIS IS A SIMPLE IMPLEMENTATION
        let count = winners.len() as u32;
        let share = self.pot / count;
        let mut remainder = self.pot % count;

        let mut results = Vec::new();

        for winner_id in winners {
            let mut payout = share;
            if remainder > 0 {
                payout += 1;
                remainder -= 1;
            }

            if let Some(player) = self.table.get_player_mut(winner_id) {
                player.chips += payout;
                results.push((winner_id, payout));
            }
        }

        self.pot = 0;
        results
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
    pub fn get_game_type(&self) -> GameType {
        match self {
            Game::FiveCardDraw(_) => GameType::FiveCardDraw,
            Game::SevenCardStud(_) => GameType::SevenCardStud,
            Game::TexasHoldEm(_) => GameType::TexasHoldEm,
        }
    }

    pub fn get_game_id(&self) -> Uuid {
        match self {
            Game::FiveCardDraw(game) => game.game_id,
            Game::SevenCardStud(game) => game.game_id,
            Game::TexasHoldEm(game) => game.game_id,
        }
    }

    pub fn get_player_count(&self) -> usize {
        match self {
            Game::FiveCardDraw(game) => game.core.table.get_player_count(),
            Game::SevenCardStud(game) => game.core.table.get_player_count(),
            Game::TexasHoldEm(game) => game.core.table.get_player_count(),
        }
    }
    pub fn get_players(&self) -> Vec<Player> { 
        match self { 
            Game::FiveCardDraw(game) => 
                game.core.table.players.clone(), 
            Game::SevenCardStud(game) => 
                game.core.table.players.clone(), 
            Game::TexasHoldEm(game) => 
                game.core.table.players.clone(), 
        }
    }

    pub fn get_betting_state(&self) -> BettingState {
        match self { 
            Game::FiveCardDraw(game) => 
                game.core.betting_state.clone(), 
            Game::SevenCardStud(game) => 
                game.core.betting_state.clone(), 
            Game::TexasHoldEm(game) => 
                game.core.betting_state.clone(), 
        }
    }

    pub fn get_betting_round(&self) -> BettingRound {
        match self { 
            Game::FiveCardDraw(game) => 
                game.betting_round.clone(), 
            Game::SevenCardStud(game) => 
                game.betting_round.clone(), 
            Game::TexasHoldEm(game) => 
                game.betting_round.clone(), 
        }
    }

    pub fn get_action_on(&self) -> Option<Uuid> { 
        match self { 
            Game::FiveCardDraw(game) => 
                game.core.action_on.clone(), 
            Game::SevenCardStud(game) => 
                game.core.action_on.clone(), 
            Game::TexasHoldEm(game) => 
                game.core.action_on.clone(), 
        }
    }

    /// Dealer index for building game state (e.g. who is dealer for display).
    pub fn get_dealer_index(&self) -> usize {
        match self {
            Game::FiveCardDraw(game) => game.core.dealer_idx,
            Game::SevenCardStud(game) => game.core.dealer_idx,
            Game::TexasHoldEm(game) => game.core.dealer_idx,
        }
    }

    /// Starts a new hand (Five Card Draw only for now). Deals 5 cards to each player and sets PreDraw.
    pub fn start_hand(&mut self) -> Result<(), String> {
        match self {
            Game::FiveCardDraw(game) => game.start_hand(),
            Game::SevenCardStud(_) | Game::TexasHoldEm(_) => {
                Err("start_hand only supported for Five Card Draw".to_string())
            }
        }
    }
    pub fn get_max_players(&self) -> usize {
        match self {
            Game::FiveCardDraw(game) => game.core.table.max_players,
            Game::SevenCardStud(game) => game.core.table.max_players,
            Game::TexasHoldEm(game) => game.core.table.max_players,
        }
    }

    pub fn get_pot(&self) -> u32 {
        match self {
            Game::FiveCardDraw(game) => game.core.pot,
            Game::SevenCardStud(game) => game.core.pot,
            Game::TexasHoldEm(game) => game.core.pot,
        }
    }

    /// Last hand result (winner names and amounts) for Five Card Draw; None for other variants or before any showdown.
    pub fn get_last_showdown(&self) -> Option<Vec<(String, u32)>> {
        match self {
            Game::FiveCardDraw(game) => game.last_showdown.clone(),
            _ => None,
        }
    }

    pub fn get_status(&self) -> GameStatus {
        match self {
            Game::FiveCardDraw(game) => {
                if game.betting_round == BettingRound::PreDeal {
                    GameStatus::WaitingForPlayers
                } else {
                    GameStatus::InProgress
                }
            }
            Game::SevenCardStud(game) => {
                if game.betting_round == BettingRound::PreDeal {
                    GameStatus::WaitingForPlayers
                } else {
                    GameStatus::InProgress
                }
            }
            Game::TexasHoldEm(game) => {
                if game.betting_round == BettingRound::PreDeal {
                    GameStatus::WaitingForPlayers
                } else {
                    GameStatus::InProgress
                }
            }
        }
    }

    pub fn is_full(&self) -> bool {
        match self {
            Game::FiveCardDraw(game) => game.core.table.is_full(),
            Game::SevenCardStud(game) => game.core.table.is_full(),
            Game::TexasHoldEm(game) => game.core.table.is_full(),
        }
    }

    pub fn is_empty(&self) -> bool { 
        match self { 
            Game::FiveCardDraw(game) => game.core.table.is_empty(), 
            Game::SevenCardStud(game) => game.core.table.is_empty(), 
            Game::TexasHoldEm(game) => game.core.table.is_empty(), 
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
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

    pub fn remove_player(&mut self, player_id: Uuid) -> Result<(), String> {
        match self {
            Game::FiveCardDraw(game) => {
                game.core.table.remove_player_from_table(player_id)
                    .map_err(|e| e.to_string())?;
                // If a hand is in progress and only 1 player remains, that player auto-wins.
                if game.betting_round != BettingRound::PreDeal
                    && game.core.table.players.len() == 1
                {
                    let winner_id = game.core.table.players[0].id;
                    let winner_name = game.core.table.players[0].username.clone();
                    let pot_amount = game.core.pot;
                    if let Some(player) = game.core.table.get_player_mut(winner_id) {
                        player.chips += pot_amount;
                    }
                    game.last_showdown = Some(vec![(winner_name, pot_amount)]);
                    game.core.pot = 0;
                    game.core.reset_for_new_hand();
                    game.betting_round = BettingRound::PreDeal;
                }
                Ok(())
            }
            Game::SevenCardStud(game) => {
                game.core.table.remove_player_from_table(player_id)
                    .map_err(|e| e.to_string())?;
                if game.betting_round != BettingRound::PreDeal
                    && game.core.table.players.len() == 1
                {
                    let winner_id = game.core.table.players[0].id;
                    let pot_amount = game.core.pot;
                    if let Some(player) = game.core.table.get_player_mut(winner_id) {
                        player.chips += pot_amount;
                    }
                    game.core.pot = 0;
                    game.core.reset_for_new_hand();
                    game.betting_round = BettingRound::PreDeal;
                }
                Ok(())
            }
            Game::TexasHoldEm(game) => {
                game.core.table.remove_player_from_table(player_id)
                    .map_err(|e| e.to_string())?;
                if game.betting_round != BettingRound::PreDeal
                    && game.core.table.players.len() == 1
                {
                    let winner_id = game.core.table.players[0].id;
                    let pot_amount = game.core.pot;
                    if let Some(player) = game.core.table.get_player_mut(winner_id) {
                        player.chips += pot_amount;
                    }
                    game.core.pot = 0;
                    game.core.reset_for_new_hand();
                    game.betting_round = BettingRound::PreDeal;
                }
                Ok(())
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
    /// Winner username and amount won from last hand; cleared when next hand starts.
    pub last_showdown: Option<Vec<(String, u32)>>,
    /// Players who have already drawn this round (one draw per player per hand).
    pub drawn_this_round: Vec<Uuid>,
}

impl FiveCardDraw {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(5),
            betting_round: BettingRound::PreDeal,
            last_showdown: None,
            drawn_this_round: Vec::new(),
        }
    }

    /// Deals 5 cards to each player and transitions to PreDraw betting.
    pub fn start_hand(&mut self) -> Result<(), String> {
        if self.betting_round != BettingRound::PreDeal {
            return Err("hand_already_started".to_string());
        }
        if self.core.table.players.len() < 2 {
            return Err("need_at_least_2_players".to_string());
        }
        self.last_showdown = None; // clear previous hand result
        self.core.deck.shuffle();
        for player in &mut self.core.table.players {
            let cards = self.core.deck.deal(5);
            if cards.len() != 5 {
                return Err("not_enough_cards".to_string());
            }
            for card in cards {
                player.hand.add(card);
            }
        }
        self.betting_round = BettingRound::PreDraw;
        self.core.action_on = None;
        if !self.core.advance_action(false) {
            return Err("no_active_players".to_string());
        }
        Ok(())
    }

    pub fn predraw_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), String> {
        if self.betting_round != BettingRound::PreDraw {
            return Err("wrong_phase".to_string());
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        // Check if only one player remains (everyone else folded/left)
        if self.core.count_active_players() == 1 {
            let active_players: Vec<_> = self.core.table.players.iter()
                .filter(|p| !p.is_folded)
                .collect();

            if let Some(winner) = active_players.first() {
                let winner_id = winner.id;
                let winner_name = winner.username.clone();
                let pot_amount = self.core.pot;
                
                if let Some(player) = self.core.table.get_player_mut(winner_id) {
                    player.chips += pot_amount;
                }
                
                // Set showdown message and reset
                self.last_showdown = Some(vec![(winner_name, pot_amount)]);
                self.core.pot = 0;
                self.core.reset_for_new_hand();
                self.betting_round = BettingRound::PreDeal;
            }
            return Ok(());
        }

        if round_over {
            self.transition_to_draw_phase();
        }

        Ok(())
    }

    /// Same logic as predraw, just different phase check and transition
    pub fn postdraw_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), String> {
        if self.betting_round != BettingRound::PostDraw {
            return Err("wrong_phase".to_string());
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        // Check if only one player remains (everyone else folded/left)
        if self.core.count_active_players() == 1 {
            let active_players: Vec<_> = self.core.table.players.iter()
                .filter(|p| !p.is_folded)
                .collect();

            if let Some(winner) = active_players.first() {
                let winner_id = winner.id;
                let winner_name = winner.username.clone();
                let pot_amount = self.core.pot;
                
                if let Some(player) = self.core.table.get_player_mut(winner_id) {
                    player.chips += pot_amount;
                }
                
                // Set showdown message and reset
                self.last_showdown = Some(vec![(winner_name, pot_amount)]);
                self.core.pot = 0;
                self.core.reset_for_new_hand();
                self.betting_round = BettingRound::PreDeal;
            }
            return Ok(());
        }

        if round_over {
            self.transition_to_showdown();
        }

        Ok(())
    }

    fn transition_to_draw_phase(&mut self) {
        self.core.reset_current_bets();
        self.betting_round = BettingRound::Drawing;
        self.core.action_on = None;
        self.drawn_this_round.clear();

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

        let results = self.core.resolve_showdown();
        // Store winner names and amounts for client to show (before reset clears table state)
        self.last_showdown = Some(
            results
                .iter()
                .map(|(uid, amt)| {
                    let name = self
                        .core
                        .table
                        .players
                        .iter()
                        .find(|p| p.id == *uid)
                        .map(|p| p.username.clone())
                        .unwrap_or_else(|| uid.to_string());
                    (name, *amt)
                })
                .collect(),
        );

        self.core.reset_for_new_hand();

        // loop back to PreDeal waiting for next start
        self.betting_round = BettingRound::PreDeal;
    }

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
        // Five Card Draw: each player gets exactly one draw per hand
        if self.drawn_this_round.contains(&player_id) {
            return Err("already_drew_this_round".to_string());
        }

        if discard_indices.len() > 5 {
            return Err("too_many_discards".to_string());
        }

        let mut unique_indices = discard_indices.clone();
        unique_indices.sort_unstable();
        unique_indices.dedup();
        let count = unique_indices.len();

        let new_cards = self.core.deck.deal(count);
        let player = self.core.table.get_player_mut(player_id).unwrap();
        player.draw(&discard_indices, new_cards)?;

        self.drawn_this_round.push(player_id);

        // Next: find a player who hasn't drawn yet; if everyone has drawn, go to post-draw
        let start_idx = self.core.get_player_index(player_id).unwrap();
        let count_players = self.core.table.players.len();
        let mut next_action = None;
        for i in 1..=count_players {
            let idx = (start_idx + i) % count_players;
            let p = &self.core.table.players[idx];
            if p.is_folded {
                continue;
            }
            if !self.drawn_this_round.contains(&p.id) {
                next_action = Some(p.id);
                break;
            }
        }
        match next_action {
            Some(uid) => {
                self.core.action_on = Some(uid);
            }
            None => {
                self.transition_to_post_draw();
            }
        }

        Ok(())
    }
}

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
            core: SharedGameState::new(7),
            betting_round: BettingRound::PreDeal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TexasHoldEm {
    pub game_id: Uuid,
    pub core: SharedGameState,
    pub betting_round: BettingRound,
}

impl TexasHoldEm {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(9),
            betting_round: BettingRound::PreDeal,
        }
    }
}
