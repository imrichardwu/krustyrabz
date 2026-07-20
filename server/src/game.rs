use crate::betting::{BettingRound, BettingState};
use crate::deck::Deck;
use crate::error::GameError;
use crate::player::Player;
use crate::table::Table;
use poker_core::{Card, CardType, DeckTrait, GameStatus, GameType};
use std::time::{Duration, Instant};
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

    // Timeout tracking
    pub action_started_at: Option<Instant>,

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
            action_started_at: None,
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
                self.action_started_at = Some(Instant::now());
                return true;
            }
        }

        self.action_on = None;
        self.action_started_at = None;
        return false;
    }

    pub fn post_bet(&mut self, player_id: Uuid, amount: u32) -> Result<(), GameError> {
        // Lookup player
        let player = self
            .table
            .get_player_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if amount > player.chips {
            return Err(GameError::InsufficientFunds);
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
    ) -> Result<bool, GameError> {
        if self.action_on != Some(player_id) {
            return Err(GameError::NotYourTurn);
        }

        let player_idx = self.get_player_index(player_id).unwrap();
        let current_contribution = self.table.players[player_idx].current_bet;
        let to_call = self.betting_state.to_call;

        match action {
            poker_core::protocol::GameAction::Fold => {
                let player = self.table.get_player_mut(player_id).unwrap();
                player.is_folded = true;
            }

            poker_core::protocol::GameAction::Check | poker_core::protocol::GameAction::Pass => {
                if current_contribution < to_call {
                    return Err(GameError::CannotCheckMustCall);
                }
            }

            poker_core::protocol::GameAction::Call => {
                let needed = to_call.saturating_sub(current_contribution);
                self.post_bet(player_id, needed)?;
            }

            poker_core::protocol::GameAction::Bet { amount } => {
                if to_call > 0 {
                    return Err(GameError::CannotBetMustRaise);
                }
                if amount < self.betting_state.min_raise {
                    return Err(GameError::BetTooSmall);
                }

                // Bet 10 means Raise To 10 (from 0)
                self.post_bet(player_id, amount)?;

                self.betting_state.to_call = amount;
                self.betting_state.last_aggressor = Some(player_id);
            }

            poker_core::protocol::GameAction::Raise { amount } => {
                if to_call == 0 {
                    return Err(GameError::CannotRaiseMustBet);
                }
                if amount < to_call + self.betting_state.min_raise {
                    return Err(GameError::RaiseTooSmall);
                }

                // "Raise To 50", in for 10, pay 40 more
                let delta = amount.saturating_sub(current_contribution);
                self.post_bet(player_id, delta)?;

                self.betting_state.to_call = amount;
                self.betting_state.raises_used += 1;
                self.betting_state.last_aggressor = Some(player_id);
            }

            _ => return Err(GameError::InvalidActionForBettingPhase),
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

    pub fn set_game_id(&mut self, game_id: Uuid) {
        match self {
            Game::FiveCardDraw(game) => game.game_id = game_id,
            Game::SevenCardStud(game) => game.game_id = game_id,
            Game::TexasHoldEm(game) => game.game_id = game_id,
        }
    }

    pub fn get_game_core(&self) -> SharedGameState {
        match self {
            Game::FiveCardDraw(game) => game.core.clone(),
            Game::SevenCardStud(game) => game.core.clone(),
            Game::TexasHoldEm(game) => game.core.clone(),
        }
    }

    pub fn set_game_core(&mut self, core: SharedGameState) {
        match self {
            Game::FiveCardDraw(game) => game.core = core,
            Game::SevenCardStud(game) => game.core = core,
            Game::TexasHoldEm(game) => game.core = core,
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
            Game::FiveCardDraw(game) => game.core.table.players.clone(),
            Game::SevenCardStud(game) => game.core.table.players.clone(),
            Game::TexasHoldEm(game) => game.core.table.players.clone(),
        }
    }

    pub fn get_betting_state(&self) -> BettingState {
        match self {
            Game::FiveCardDraw(game) => game.core.betting_state.clone(),
            Game::SevenCardStud(game) => game.core.betting_state.clone(),
            Game::TexasHoldEm(game) => game.core.betting_state.clone(),
        }
    }

    pub fn get_swap_flag(&self) -> bool {
        match self {
            Game::FiveCardDraw(game) => game.swap_flag.clone(),
            Game::SevenCardStud(game) => game.swap_flag.clone(),
            Game::TexasHoldEm(game) => game.swap_flag.clone(),
        }
    }

    pub fn set_swap_flag(&mut self, flag: bool) {
        match self {
            Game::FiveCardDraw(game) => game.swap_flag = flag,
            Game::SevenCardStud(game) => game.swap_flag = flag,
            Game::TexasHoldEm(game) => game.swap_flag = flag,
        }
    }

    pub fn get_betting_round(&self) -> BettingRound {
        match self {
            Game::FiveCardDraw(game) => game.betting_round.clone(),
            Game::SevenCardStud(game) => game.betting_round.clone(),
            Game::TexasHoldEm(game) => game.betting_round.clone(),
        }
    }

    pub fn get_action_on(&self) -> Option<Uuid> {
        match self {
            Game::FiveCardDraw(game) => game.core.action_on.clone(),
            Game::SevenCardStud(game) => game.core.action_on.clone(),
            Game::TexasHoldEm(game) => game.core.action_on.clone(),
        }
    }

    pub fn get_action_started_at(&self) -> Option<std::time::Instant> {
        match self {
            Game::FiveCardDraw(game) => game.core.action_started_at,
            Game::SevenCardStud(game) => game.core.action_started_at,
            Game::TexasHoldEm(game) => game.core.action_started_at,
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

    /// Starts a new hand for any supported game variant.
    pub fn start_hand(&mut self) -> Result<(), GameError> {
        match self {
            Game::FiveCardDraw(game) => game.start_hand(),
            Game::SevenCardStud(game) => game.start_hand(),
            Game::TexasHoldEm(game) => game.start_hand(),
        }
    }

    /// Dispatches a player action to the appropriate game variant.
    pub fn handle_action(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), GameError> {
        match self {
            Game::FiveCardDraw(_) => Err(GameError::UsePerActionHandlers),
            Game::SevenCardStud(game) => game.handle_action(player_id, action),
            Game::TexasHoldEm(game) => game.handle_action(player_id, action),
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

    /// Last hand result (winner names and amounts); None before any showdown.
    pub fn get_last_showdown(&self) -> Option<Vec<(String, u32)>> {
        match self {
            Game::FiveCardDraw(game) => game.last_showdown.clone(),
            Game::SevenCardStud(game) => game.last_showdown.clone(),
            Game::TexasHoldEm(game) => game.last_showdown.clone(),
        }
    }

    /// Community cards (Texas Hold'em only); empty for other variants.
    pub fn get_community_cards(&self) -> Vec<String> {
        match self {
            Game::TexasHoldEm(game) => game.community_cards.iter().map(|c| c.to_string()).collect(),
            _ => Vec::new(),
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

    /// Marks a player as sitting out for the next hand (Seven Card Stud only).
    pub fn sit_out_player(&mut self, player_id: Uuid) -> Result<(), GameError> {
        match self {
            Game::SevenCardStud(game) => game.sit_out_player(player_id),
            _ => Err(GameError::SitOutOnlySevenCardStud),
        }
    }

    pub fn add_viewer(&mut self, viewer_id: Uuid) {
        match self {
            Game::FiveCardDraw(game) => game.core.table.add_viewer(viewer_id),
            Game::SevenCardStud(game) => game.core.table.add_viewer(viewer_id),
            Game::TexasHoldEm(game) => game.core.table.add_viewer(viewer_id),
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        match self {
            Game::FiveCardDraw(game) => game
                .core
                .table
                .seat_player(player)
                .map_err(|e| GameError::Table(e.to_string())),
            Game::SevenCardStud(game) => game
                .core
                .table
                .seat_player(player)
                .map_err(|e| GameError::Table(e.to_string())),
            Game::TexasHoldEm(game) => game
                .core
                .table
                .seat_player(player)
                .map_err(|e| GameError::Table(e.to_string())),
        }
    }

    /// Check if the current player's turn has timed out (30 seconds).
    /// Returns Some(player_id) if timeout occurred, None otherwise.
    pub fn check_timeout(&self, timeout_duration: Duration) -> Option<Uuid> {
        if let (Some(player_id), Some(started_at)) =
            (self.get_action_on(), self.get_action_started_at())
        {
            if started_at.elapsed() > timeout_duration {
                return Some(player_id);
            }
        }
        None
    }

    /// Auto-fold a player due to timeout.
    pub fn timeout_player(&mut self, player_id: Uuid) -> Result<(), GameError> {
        use poker_core::protocol::GameAction;

        match self {
            Game::FiveCardDraw(game) => {
                // Auto-fold the player
                match game.betting_round {
                    BettingRound::PreDraw => game.predraw_betting(player_id, GameAction::Fold),
                    BettingRound::PostDraw => game.postdraw_betting(player_id, GameAction::Fold),
                    _ => Err(GameError::NotInBettingPhase),
                }
            }
            Game::SevenCardStud(game) => {
                // Auto-fold the player
                if game.betting_round != BettingRound::PreDeal {
                    game.handle_action(player_id, GameAction::Fold)
                } else {
                    Err(GameError::NotInBettingPhase)
                }
            }
            Game::TexasHoldEm(game) => {
                // Auto-fold the player
                if game.betting_round != BettingRound::PreDeal {
                    game.handle_action(player_id, GameAction::Fold)
                } else {
                    Err(GameError::NotInBettingPhase)
                }
            }
        }
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> Result<(), GameError> {
        match self {
            Game::FiveCardDraw(game) => {
                game.core
                    .table
                    .remove_player_from_table(player_id)
                    .map_err(|e| GameError::Table(e.to_string()))?;
                // If a hand is in progress and only 1 player remains, that player auto-wins.
                if game.betting_round != BettingRound::PreDeal && game.core.table.players.len() == 1
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
                game.core
                    .table
                    .remove_player_from_table(player_id)
                    .map_err(|e| GameError::Table(e.to_string()))?;
                if game.betting_round != BettingRound::PreDeal && game.core.table.players.len() == 1
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
                game.core
                    .table
                    .remove_player_from_table(player_id)
                    .map_err(|e| GameError::Table(e.to_string()))?;
                if game.betting_round != BettingRound::PreDeal && game.core.table.players.len() == 1
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

#[derive(Debug, Clone)]
pub struct FiveCardDraw {
    pub game_id: Uuid,
    pub core: SharedGameState,
    pub betting_round: BettingRound,
    /// Winner username and amount won from last hand; cleared when next hand starts.
    pub last_showdown: Option<Vec<(String, u32)>>,
    /// Players who have already drawn this round (one draw per player per hand).
    pub drawn_this_round: Vec<Uuid>,
    /// Dealer's choice flag, signals the client that the game type has changed
    pub swap_flag: bool,
}

impl FiveCardDraw {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(5),
            betting_round: BettingRound::PreDeal,
            last_showdown: None,
            drawn_this_round: Vec::new(),
            swap_flag: false,
        }
    }

    /// Deals 5 cards to each player and transitions to PreDraw betting.
    pub fn start_hand(&mut self) -> Result<(), GameError> {
        if self.betting_round != BettingRound::PreDeal {
            return Err(GameError::HandAlreadyStarted);
        }
        if self.core.table.players.len() < 2 {
            return Err(GameError::NeedAtLeast2Players);
        }
        self.last_showdown = None; // clear previous hand result
        self.core.deck.shuffle();
        for player in &mut self.core.table.players {
            let cards = self.core.deck.deal(5);
            if cards.len() != 5 {
                return Err(GameError::NotEnoughCards);
            }
            for card in cards {
                player.hand.add(card);
            }
        }
        self.betting_round = BettingRound::PreDraw;
        self.core.action_on = None;
        if !self.core.advance_action(false) {
            return Err(GameError::NoActivePlayers);
        }
        Ok(())
    }

    pub fn predraw_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), GameError> {
        if self.betting_round != BettingRound::PreDraw {
            return Err(GameError::WrongPhase);
        }

        if matches!(action, poker_core::protocol::GameAction::Pass) {
            return Err(GameError::CannotPassFirstRound);
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        // Check if only one player remains (everyone else folded/left)
        if self.core.count_active_players() == 1 {
            let active_players: Vec<_> = self
                .core
                .table
                .players
                .iter()
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
    ) -> Result<(), GameError> {
        if self.betting_round != BettingRound::PostDraw {
            return Err(GameError::WrongPhase);
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        // Check if only one player remains (everyone else folded/left)
        if self.core.count_active_players() == 1 {
            let active_players: Vec<_> = self
                .core
                .table
                .players
                .iter()
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
    ) -> Result<(), GameError> {
        if self.core.action_on != Some(player_id) {
            return Err(GameError::NotYourTurn);
        }
        if self.betting_round != BettingRound::Drawing {
            return Err(GameError::WrongPhaseExpectingDrawing);
        }
        // Five Card Draw: each player gets exactly one draw per hand
        if self.drawn_this_round.contains(&player_id) {
            return Err(GameError::AlreadyDrewThisRound);
        }

        if discard_indices.len() > 5 {
            return Err(GameError::TooManyDiscards);
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
    pub last_showdown: Option<Vec<(String, u32)>>,
    pub sitting_out: Vec<Uuid>,
    pub swap_flag: bool,
}

impl SevenCardStud {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(7),
            betting_round: BettingRound::PreDeal,
            last_showdown: None,
            sitting_out: Vec::new(),
            swap_flag: false,
        }
    }

    /// Mark a player as sitting out for the next hand (opt out of ante).
    /// Only allowed when no hand is in progress.
    pub fn sit_out_player(&mut self, player_id: Uuid) -> Result<(), GameError> {
        if self.betting_round != BettingRound::PreDeal {
            return Err(GameError::SitOutBeforeHandStarts);
        }
        if self.core.table.get_player(player_id).is_none() {
            return Err(GameError::PlayerNotFoundAtTable);
        }
        if !self.sitting_out.contains(&player_id) {
            self.sitting_out.push(player_id);
        }
        Ok(())
    }

    /// Cancel a sit-out (player decides to play after all).
    pub fn cancel_sit_out(&mut self, player_id: Uuid) {
        self.sitting_out.retain(|&id| id != player_id);
    }

    fn get_suit_val(suit: poker_core::Suit) -> u8 {
        match suit {
            poker_core::Suit::Clubs => 1,
            poker_core::Suit::Diamonds => 2,
            poker_core::Suit::Hearts => 3,
            poker_core::Suit::Spades => 4,
        }
    }

    fn find_lowest(&self) -> Option<Uuid> {
        let mut lowest: Option<(u8, u8, Uuid)> = None;

        for p in &self.core.table.players {
            if p.is_folded {
                continue;
            }
            for c in p.hand.cards() {
                if c.card_type == poker_core::CardType::Up {
                    let r_val = c.rank as u8;
                    let s_val = Self::get_suit_val(c.suit);

                    match lowest {
                        None => lowest = Some((r_val, s_val, p.id)),

                        Some((low_r, low_s, _)) => {
                            if r_val < low_r || (low_r == r_val && s_val < low_s) {
                                lowest = Some((r_val, s_val, p.id));
                            }
                        }
                    }
                }
            }
        }
        lowest.map(|(_, _, id)| id)
    }

    fn eval_best_showing_hand(&self) -> Option<Uuid> {
        let mut best_id = None;
        let mut best_rank: Option<poker_core::hand::HandRank> = None;

        for p in &self.core.table.players {
            if p.is_folded {
                continue;
            }

            let mut up_hand = poker_core::hand::Hand::new();
            for c in p.hand.cards() {
                if c.card_type == poker_core::CardType::Up {
                    up_hand.add(*c);
                }
            }

            let rank = up_hand.evaluate();
            match best_rank {
                None => {
                    best_rank = Some(rank);
                    best_id = Some(p.id);
                }
                Some(ref current) => {
                    if rank > *current {
                        best_rank = Some(rank);
                        best_id = Some(p.id);
                    }
                }
            }
        }
        best_id
    }

    /// Deals 2 private cards + 1 face-up card to each active (non-sitting-out) player
    /// and starts Third Street. Collects a 50-chip ante from each active player.
    pub fn start_hand(&mut self) -> Result<(), GameError> {
        if self.betting_round != BettingRound::PreDeal {
            return Err(GameError::HandAlreadyStarted);
        }

        // Determine which players are active (not sitting out)
        let active_ids: Vec<Uuid> = self
            .core
            .table
            .players
            .iter()
            .filter(|p| !self.sitting_out.contains(&p.id))
            .map(|p| p.id)
            .collect();

        if active_ids.len() < 2 {
            return Err(GameError::NeedAtLeast2ActivePlayers);
        }

        self.last_showdown = None;
        self.core.deck.shuffle();

        // Collect 50-chip ante from each active player
        const STUD_ANTE: u32 = 50;
        for &pid in &active_ids {
            let _ = self.core.post_bet(pid, STUD_ANTE);
        }

        // Fold sitting-out players so they are skipped in betting logic
        for player in &mut self.core.table.players {
            if self.sitting_out.contains(&player.id) {
                player.is_folded = true;
            }
        }

        // Deal 2 down + 1 up to every active player only
        for player in &mut self.core.table.players {
            if self.sitting_out.contains(&player.id) {
                continue;
            }
            let mut three = self.core.deck.deal(3);
            if three.len() < 3 {
                return Err(GameError::NotEnoughCards);
            }
            let up = three.pop().unwrap();
            player.hand.add(Card::construct(
                three[0].rank,
                three[0].suit,
                CardType::Private,
            ));
            player.hand.add(Card::construct(
                three[1].rank,
                three[1].suit,
                CardType::Private,
            ));
            player
                .hand
                .add(Card::construct(up.rank, up.suit, CardType::Up));
        }

        // Clear sit-outs — they apply only to one hand
        self.sitting_out.clear();

        self.betting_round = BettingRound::ThirdStreet;

        self.core.action_on = self.find_lowest();
        if self.core.action_on.is_none() {
            return Err(GameError::NoUpCardsFound);
        }

        Ok(())
    }

    /// Handles any betting action for the current street.
    pub fn handle_action(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), GameError> {
        match self.betting_round {
            BettingRound::PreDeal => Err(GameError::GameNotStarted),
            BettingRound::ThirdStreet
            | BettingRound::FourthStreet
            | BettingRound::FifthStreet
            | BettingRound::SixthStreet
            | BettingRound::SeventhStreet => self.street_betting(player_id, action),
            _ => Err(GameError::WrongPhase),
        }
    }

    fn street_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), GameError> {
        if self.betting_round == BettingRound::ThirdStreet
            && matches!(action, poker_core::protocol::GameAction::Pass)
        {
            return Err(GameError::CannotPassFirstRound);
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        // Last player standing wins immediately
        if self.core.count_active_players() == 1 {
            self.award_last_player();
            return Ok(());
        }

        if round_over {
            match self.betting_round {
                BettingRound::ThirdStreet => {
                    self.deal_next_street(BettingRound::FourthStreet, CardType::Up)
                }
                BettingRound::FourthStreet => {
                    self.deal_next_street(BettingRound::FifthStreet, CardType::Up)
                }
                BettingRound::FifthStreet => {
                    self.deal_next_street(BettingRound::SixthStreet, CardType::Up)
                }
                BettingRound::SixthStreet => {
                    self.deal_next_street(BettingRound::SeventhStreet, CardType::Private)
                }
                BettingRound::SeventhStreet => self.transition_to_showdown(),
                _ => {}
            }
        }
        Ok(())
    }

    /// Deals one card to every non-folded player and advances to the next street.
    fn deal_next_street(&mut self, next_round: BettingRound, card_type: CardType) {
        self.core.reset_current_bets();

        for player in &mut self.core.table.players {
            if !player.is_folded {
                if let Some(raw) = self.core.deck.deal(1).into_iter().next() {
                    player
                        .hand
                        .add(Card::construct(raw.rank, raw.suit, card_type));
                }
            }
        }

        self.betting_round = next_round;

        self.core.action_on = self.eval_best_showing_hand();

        if self.core.action_on.is_none() {
            self.transition_to_showdown();
        }
    }

    fn award_last_player(&mut self) {
        if let Some(winner) = self.core.table.players.iter().find(|p| !p.is_folded) {
            let winner_id = winner.id;
            let winner_name = winner.username.clone();
            let pot_amount = self.core.pot;
            if let Some(player) = self.core.table.get_player_mut(winner_id) {
                player.chips += pot_amount;
            }
            self.last_showdown = Some(vec![(winner_name, pot_amount)]);
            self.core.pot = 0;
            self.core.reset_for_new_hand();
            self.betting_round = BettingRound::PreDeal;
        }
    }

    fn transition_to_showdown(&mut self) {
        self.core.reset_current_bets();
        let results = self.core.resolve_showdown();
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
        self.betting_round = BettingRound::PreDeal;
    }
}

const SMALL_BLIND: u32 = 5;
const BIG_BLIND: u32 = 10;

#[derive(Debug, Clone)]
pub struct TexasHoldEm {
    pub game_id: Uuid,
    pub core: SharedGameState,
    pub betting_round: BettingRound,
    pub last_showdown: Option<Vec<(String, u32)>>,
    /// The shared community cards (flop/turn/river).
    pub community_cards: Vec<Card>,
    pub swap_flag: bool,
}

impl TexasHoldEm {
    pub fn new() -> Self {
        Self {
            game_id: Uuid::new_v4(),
            core: SharedGameState::new(9),
            betting_round: BettingRound::PreDeal,
            last_showdown: None,
            community_cards: Vec::new(),
            swap_flag: false,
        }
    }

    /// Posts blinds, deals 2 private hole cards to each player, and starts PreFlop.
    pub fn start_hand(&mut self) -> Result<(), GameError> {
        if self.betting_round != BettingRound::PreDeal {
            return Err(GameError::HandAlreadyStarted);
        }
        if self.core.table.players.len() < 2 {
            return Err(GameError::NeedAtLeast2Players);
        }
        self.last_showdown = None;
        self.community_cards.clear();
        self.core.deck.shuffle();

        let count = self.core.table.players.len();
        let sb_idx = (self.core.dealer_idx + 1) % count;
        let bb_idx = (self.core.dealer_idx + 2) % count;
        let sb_id = self.core.table.players[sb_idx].id;
        let bb_id = self.core.table.players[bb_idx].id;

        // Post blinds (best-effort; player may not have enough chips)
        let _ = self.core.post_bet(
            sb_id,
            SMALL_BLIND.min(self.core.table.players[sb_idx].chips),
        );
        let _ = self
            .core
            .post_bet(bb_id, BIG_BLIND.min(self.core.table.players[bb_idx].chips));

        // BB acts as the initial "aggressor" so PreFlop ends correctly
        self.core.betting_state.to_call = BIG_BLIND;
        self.core.betting_state.last_aggressor = Some(bb_id);

        // Deal 2 private cards to each player
        for player in &mut self.core.table.players {
            let cards = self.core.deck.deal(2);
            if cards.len() < 2 {
                return Err(GameError::NotEnoughCards);
            }
            for card in cards {
                player
                    .hand
                    .add(Card::construct(card.rank, card.suit, CardType::Private));
            }
        }

        self.betting_round = BettingRound::PreFlop;
        // Start action at UTG (player after BB)
        self.core.action_on = Some(bb_id);
        if !self.core.advance_action(false) {
            return Err(GameError::NoActivePlayers);
        }
        Ok(())
    }

    /// Handles a betting action for the current round.
    pub fn handle_action(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), GameError> {
        match self.betting_round {
            BettingRound::PreDeal => Err(GameError::GameNotStarted),
            BettingRound::PreFlop
            | BettingRound::Flop
            | BettingRound::Turn
            | BettingRound::River => self.round_betting(player_id, action),
            _ => Err(GameError::WrongPhase),
        }
    }

    fn round_betting(
        &mut self,
        player_id: Uuid,
        action: poker_core::protocol::GameAction,
    ) -> Result<(), GameError> {
        if self.betting_round == BettingRound::PreFlop
            && matches!(action, poker_core::protocol::GameAction::Pass)
        {
            return Err(GameError::CannotPassFirstRound);
        }

        let round_over = self.core.process_betting_action(player_id, action)?;

        // Last player standing wins immediately
        if self.core.count_active_players() == 1 {
            self.award_last_player();
            return Ok(());
        }

        if round_over {
            match self.betting_round {
                BettingRound::PreFlop => self.deal_flop(),
                BettingRound::Flop => self.deal_turn(),
                BettingRound::Turn => self.deal_river(),
                BettingRound::River => self.transition_to_showdown(),
                _ => {}
            }
        }
        Ok(())
    }

    fn deal_community(&mut self, count: usize, next_round: BettingRound) {
        self.core.reset_current_bets();
        let cards = self.core.deck.deal(count);
        for card in cards {
            let community = Card::construct(card.rank, card.suit, CardType::Community);
            self.community_cards.push(community);
        }
        self.betting_round = next_round;
        self.core.action_on = None;
        if !self.core.advance_action(false) {
            self.transition_to_showdown();
        }
    }

    fn deal_flop(&mut self) {
        self.deal_community(3, BettingRound::Flop);
    }
    fn deal_turn(&mut self) {
        self.deal_community(1, BettingRound::Turn);
    }
    fn deal_river(&mut self) {
        self.deal_community(1, BettingRound::River);
    }

    fn award_last_player(&mut self) {
        if let Some(winner) = self.core.table.players.iter().find(|p| !p.is_folded) {
            let winner_id = winner.id;
            let winner_name = winner.username.clone();
            let pot_amount = self.core.pot;
            if let Some(player) = self.core.table.get_player_mut(winner_id) {
                player.chips += pot_amount;
            }
            self.last_showdown = Some(vec![(winner_name, pot_amount)]);
            self.core.pot = 0;
            self.community_cards.clear();
            self.core.reset_for_new_hand();
            self.betting_round = BettingRound::PreDeal;
        }
    }

    fn transition_to_showdown(&mut self) {
        self.core.reset_current_bets();

        // Combine each player's hole cards with the 5 community cards for evaluation
        let community = self.community_cards.clone();
        for player in &mut self.core.table.players {
            if !player.is_folded {
                for &card in &community {
                    player.hand.add(card);
                }
            }
        }

        let results = self.core.resolve_showdown();
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

        self.community_cards.clear();
        self.core.reset_for_new_hand();
        self.betting_round = BettingRound::PreDeal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_core::protocol::GameAction;
    use poker_core::{Card, CardType, Rank, Suit};
    use uuid::Uuid;

    fn mock_player(name: &str, chips: u32) -> Player {
        Player {
            id: Uuid::new_v4(),
            username: name.to_string(),
            chips,
            hand: poker_core::hand::Hand::new(),
            is_folded: false,
            game_id: Default::default(),
            current_bet: 0,
        }
    }

    #[test]
    fn test_shared_state_betting_validation() {
        let mut core = SharedGameState::new(5);
        let p1 = mock_player("Alice", 100);
        let p2 = mock_player("Bob", 100);
        let id1 = p1.id;
        let id2 = p2.id;

        core.table.seat_player(p1).unwrap();
        core.table.seat_player(p2).unwrap();
        core.action_on = Some(id1);
        core.betting_state.min_raise = 10;

        // P1 Cannot check if there is a bet to call (simulated by artificially setting to_call)
        core.betting_state.to_call = 10;
        let check_err = core.process_betting_action(id1, GameAction::Check);
        assert!(check_err.is_err());
        assert_eq!(check_err.unwrap_err().to_string(), "cannot_check_must_call");

        // P1 Calls the 10
        assert!(core.process_betting_action(id1, GameAction::Call).is_ok());
        assert_eq!(core.table.get_player_mut(id1).unwrap().chips, 90);
        assert_eq!(core.pot, 10);

        // Action moved to P2. P2 tries to bet instead of raise
        let bet_err = core.process_betting_action(id2, GameAction::Bet { amount: 20 });
        assert!(bet_err.is_err());
        assert_eq!(bet_err.unwrap_err().to_string(), "cannot_bet_must_raise");

        // P2 correctly raises to 30
        assert!(
            core.process_betting_action(id2, GameAction::Raise { amount: 30 })
                .is_ok()
        );
        assert_eq!(core.pot, 40); // 10 from P1 + 30 from P2
    }

    #[test]
    fn test_shared_state_split_pot() {
        let mut core = SharedGameState::new(5);
        let mut p1 = mock_player("Alice", 0);
        let mut p2 = mock_player("Bob", 0);
        let mut p3 = mock_player("Charlie", 0);

        // Give P1 and P2 the exact same hand (Pair of Aces)
        p1.hand
            .add(Card::construct(Rank::Ace, Suit::Spades, CardType::Private));
        p1.hand
            .add(Card::construct(Rank::Ace, Suit::Hearts, CardType::Private));

        p2.hand
            .add(Card::construct(Rank::Ace, Suit::Clubs, CardType::Private));
        p2.hand.add(Card::construct(
            Rank::Ace,
            Suit::Diamonds,
            CardType::Private,
        ));

        // P3 gets a worse hand (High card King)
        p3.hand
            .add(Card::construct(Rank::King, Suit::Spades, CardType::Private));
        p3.hand
            .add(Card::construct(Rank::Two, Suit::Hearts, CardType::Private));

        let id1 = p1.id;
        let id2 = p2.id;

        core.table.seat_player(p1).unwrap();
        core.table.seat_player(p2).unwrap();
        core.table.seat_player(p3).unwrap();
        core.pot = 101; // Odd pot to test remainder distribution

        let results = core.resolve_showdown();

        // Should only have 2 winners
        assert_eq!(results.len(), 2);

        // Pot is 101. Split is 50 each. Remainder 1 goes to first winner.
        let p1_chips = core.table.get_player_mut(id1).unwrap().chips;
        let p2_chips = core.table.get_player_mut(id2).unwrap().chips;

        assert_eq!(p1_chips + p2_chips, 101);
        assert!(p1_chips == 50 || p1_chips == 51);
    }

    // Could add these if we really wanted to.

    #[test]
    fn test_seven_card_stud_rotation_rules() {
        let mut game = SevenCardStud::new();
        let mut p1 = mock_player("Alice", 100);
        let mut p2 = mock_player("Bob", 100);

        // Alice: 2 spades (higher suit), Bob: 2 clubs(lower suit)
        p1.hand
            .add(Card::construct(Rank::Two, Suit::Spades, CardType::Up));
        p2.hand
            .add(Card::construct(Rank::Two, Suit::Clubs, CardType::Up));

        let id_alice = p1.id;
        let id_bob = p2.id;

        game.core.table.seat_player(p1).unwrap();
        game.core.table.seat_player(p2).unwrap();

        // On 3rd street, lowest upcard goes first (bring-in)
        let action_uid = game.find_lowest();
        assert_eq!(action_uid, Some(id_bob));

        // Simulate 4th street by improving Alice's visible hand
        game.core
            .table
            .get_player_mut(id_alice)
            .unwrap()
            .hand
            .add(Card::construct(Rank::Two, Suit::Hearts, CardType::Up));
        game.core
            .table
            .get_player_mut(id_bob)
            .unwrap()
            .hand
            .add(Card::construct(Rank::Three, Suit::Diamonds, CardType::Up));

        // Now Alice shows a pair, so she should act first
        let action_uid_4th = game.eval_best_showing_hand();
        assert_eq!(action_uid_4th, Some(id_alice));
    }

    #[test]
    fn test_texas_holdem_community_state_machine() {
        let mut game = TexasHoldEm::new();

        // Seating order matters for blinds and action
        let p1 = mock_player("Alice", 1000); // Dealer / UTG
        let p2 = mock_player("Bob", 1000); // Small blind
        let p3 = mock_player("Charlie", 1000); // Big blind

        let id1 = p1.id;
        let id2 = p2.id;
        let id3 = p3.id;

        game.core.table.seat_player(p1).unwrap();
        game.core.table.seat_player(p2).unwrap();
        game.core.table.seat_player(p3).unwrap();

        // Start hand to blinds posted, cards dealt
        assert!(game.start_hand().is_ok());
        assert_eq!(game.betting_round, BettingRound::PreFlop);
        assert_eq!(game.core.pot, 15);

        // First action is UTG (left of big blind)
        assert_eq!(game.core.action_on, Some(id1));

        assert!(game.handle_action(id1, GameAction::Call).is_ok());

        // Small blind needs to complete to match big blind
        assert_eq!(game.core.action_on, Some(id2));

        // In this engine, once SB calls, round ends immediately
        assert!(game.handle_action(id2, GameAction::Call).is_ok());

        assert_eq!(game.betting_round, BettingRound::Flop);
        assert_eq!(game.community_cards.len(), 3);
        assert_eq!(game.core.pot, 30);

        // Post-flop starts from small blind
        assert_eq!(game.core.action_on, Some(id2));

        assert!(
            game.handle_action(id2, GameAction::Bet { amount: 50 })
                .is_ok()
        );
        assert!(game.handle_action(id3, GameAction::Fold).is_ok());
        assert!(game.handle_action(id1, GameAction::Call).is_ok());

        assert_eq!(game.betting_round, BettingRound::Turn);
        assert_eq!(game.community_cards.len(), 4);

        assert_eq!(game.core.action_on, Some(id2));
        assert!(game.handle_action(id2, GameAction::Check).is_ok());
        assert!(game.handle_action(id1, GameAction::Check).is_ok());

        // River dealt
        assert_eq!(game.betting_round, BettingRound::River);
        assert_eq!(game.community_cards.len(), 5);

        // Double check to showdown
        assert!(game.handle_action(id2, GameAction::Check).is_ok());
        assert!(game.handle_action(id1, GameAction::Check).is_ok());

        // After showdown, everything should be reset
        assert_eq!(game.betting_round, BettingRound::PreDeal);
        assert_eq!(game.community_cards.len(), 0);
        assert_eq!(game.core.pot, 0);
        assert!(game.last_showdown.is_some());
    }
}
