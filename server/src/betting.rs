use std::collections::{HashMap, HashSet};
use strum_macros::EnumString; 
use uuid::Uuid;

//stores round-types for all 3 poker variants
//PreDeal is shared across all 3 variants, it specifies that the game has not yet started
//
//TODO add convenience comments mapping rounds to variants and explaining them
#[repr(u8)]
#[derive(Debug, Display)]
pub enum BettingRound {
    #[strum(serialize = "predeal")] PreDeal, 
    #[strum(serialize = "predeal")] PreDeal,
    #[strum(serialize = "predraw")] PreDraw, 
    #[strum(serialiez = "postdraw")] PostDraw, 
    #[strum(serialize = "preflop")] PreFlop, 
    #[strum(serialize = "flop")] Flop, 
    #[strum(serialize = "turn")] Turn, 
    #[strum(serialize = "river")] River, 
    #[strum(serialize = "third_street")] ThirdStreet, 
    #[strum(serialize = "fourth_street")] FourthStreet, 
    #[strum(serialize = "fifth_street")] FifthStreet, 
    #[strum(serialize = "sixth_street")] SixthStreet, 
    #[strum(serialize = "seventh_street")] SeventhStreet
}

#[derive(Debug)] 
pub struct BettingState {
    pub pot: u32,
    pub to_call: u32,
    pub min_bet: u32,
    pub max_bet: u32,

    pub raises_used: u8,
    pub max_raises: u8,

    pub contributions: HashMap<Uuid, u32>, // per player, this round
    pub folded: HashSet<Uuid>,
    pub betting_round: BettingRounds
} 
