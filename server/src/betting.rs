// betting.rs - Betting round and state management
//
// This module handles betting rounds and state for all poker variants.

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Stores round-types for all 3 poker variants.
/// PreDeal is shared across all 3 variants, it specifies that the game has not yet started.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum BettingRound {
    PreDeal,
    // Five Card Draw rounds
    PreDraw,
    PostDraw,
    // Texas Hold'Em rounds
    PreFlop,
    Flop,
    Turn,
    River,
    // Seven Card Stud rounds
    ThirdStreet,
    FourthStreet,
    FifthStreet,
    SixthStreet,
    SeventhStreet,
}

impl std::fmt::Display for BettingRound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BettingRound::PreDeal => write!(f, "predeal"),
            BettingRound::PreDraw => write!(f, "predraw"),
            BettingRound::PostDraw => write!(f, "postdraw"),
            BettingRound::PreFlop => write!(f, "preflop"),
            BettingRound::Flop => write!(f, "flop"),
            BettingRound::Turn => write!(f, "turn"),
            BettingRound::River => write!(f, "river"),
            BettingRound::ThirdStreet => write!(f, "third_street"),
            BettingRound::FourthStreet => write!(f, "fourth_street"),
            BettingRound::FifthStreet => write!(f, "fifth_street"),
            BettingRound::SixthStreet => write!(f, "sixth_street"),
            BettingRound::SeventhStreet => write!(f, "seventh_street"),
        }
    }
}

/// Tracks the current betting state within a round.
#[derive(Debug, Clone)]
pub struct BettingState {
    pub pot: u32,
    pub to_call: u32,
    pub min_bet: u32,
    pub max_bet: u32,
    pub raises_used: u8,
    pub max_raises: u8,
    pub contributions: HashMap<Uuid, u32>,
    pub folded: HashSet<Uuid>,
    pub betting_round: BettingRound,
}

impl BettingState {
    pub fn new() -> Self {
        Self {
            pot: 0,
            to_call: 0,
            min_bet: 10,
            max_bet: 100,
            raises_used: 0,
            max_raises: 3,
            contributions: HashMap::new(),
            folded: HashSet::new(),
            betting_round: BettingRound::PreDeal,
        }
    }

    pub fn with_limits(min_bet: u32, max_bet: u32, max_raises: u8) -> Self {
        Self {
            pot: 0,
            to_call: 0,
            min_bet,
            max_bet,
            raises_used: 0,
            max_raises,
            contributions: HashMap::new(),
            folded: HashSet::new(),
            betting_round: BettingRound::PreDeal,
        }
    }

    pub fn reset_round(&mut self) {
        self.to_call = 0;
        self.raises_used = 0;
        self.contributions.clear();
    }

    pub fn add_to_pot(&mut self, player_id: Uuid, amount: u32) {
        self.pot += amount;
        *self.contributions.entry(player_id).or_insert(0) += amount;
    }

    pub fn fold_player(&mut self, player_id: Uuid) {
        self.folded.insert(player_id);
    }

    pub fn is_folded(&self, player_id: &Uuid) -> bool {
        self.folded.contains(player_id)
    }

    pub fn get_contribution(&self, player_id: &Uuid) -> u32 {
        *self.contributions.get(player_id).unwrap_or(&0)
    }

    pub fn advance_round(&mut self, next_round: BettingRound) {
        self.betting_round = next_round;
        self.reset_round();
    }
}

impl Default for BettingState {
    fn default() -> Self {
        Self::new()
    }
}
