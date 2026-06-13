// betting.rs - Betting round and state management
//
// This module handles betting rounds and state for all poker variants.

use uuid::Uuid;
use strum_macros::Display;

/// Stores round-types for all 3 poker variants.
/// PreDeal is shared across all 3 variants, it specifies that the game has not yet started.
#[derive(Debug, Clone, PartialEq, Copy, Display)]
pub enum BettingRound {
    #[strum(serialize = "predeal")]
    PreDeal,
    // Five Card Draw rounds
    #[strum(serialize = "predraw")]
    PreDraw,
    #[strum(serialize = "postdraw")]
    PostDraw,
    #[strum(serialize = "drawing")]
    Drawing,
    // Texas Hold'Em rounds
    #[strum(serialize = "preflop")]
    PreFlop,
    #[strum(serialize = "flop")]
    Flop,
    #[strum(serialize = "turn")]
    Turn,
    #[strum(serialize = "river")]
    River,
    // Seven Card Stud rounds
    #[strum(serialize = "third_street")]
    ThirdStreet,
    #[strum(serialize = "fourth_street")]
    FourthStreet,
    #[strum(serialize = "fifth_street")]
    FifthStreet,
    #[strum(serialize = "sixth_street")]
    SixthStreet,
    #[strum(serialize = "seventh_street")]
    SeventhStreet,
}

/// Represents an action taken by a player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BetAction {
    Check,
    Fold,
    Call,
   //Raise To (the total amount the player wishes to reach)
    RaiseTo(u32), 
    AllIn,
}

/// Tracks the constraints and metadata of the current betting round.
#[derive(Debug, Clone)]
pub struct BettingState {
    pub to_call: u32,
    pub min_raise: u32,
    pub raises_used: u8,
    pub max_raises: u8,
    
    // Flow Control
    pub last_aggressor: Option<Uuid>, 
}

impl BettingState {
    pub fn new() -> Self {
        Self {
            to_call: 0,
            min_raise: 10,
            raises_used: 0,
            max_raises: 3,
            last_aggressor: None,
        }
    }

    pub fn with_limits(min_raise: u32, max_raises: u8) -> Self {
        Self {
            to_call: 0,
            min_raise,
            raises_used: 0,
            max_raises,
            last_aggressor: None,
        }
    }

    pub fn reset_round(&mut self) {
        self.to_call = 0;
        self.raises_used = 0;
        self.last_aggressor = None;
    }
}

impl Default for BettingState {
    fn default() -> Self {
        Self::new()
    }
}
