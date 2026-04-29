use poker_core::{Deck,Player, BettingRounds, BettingState, Table}; 
use strum_macros::EnumString; 
use std::sync::{Arc, Mutex}; 
use std::str::FromStr; 
use std::vec::vec; 
use uuid::Uuid; 

//TODO Hoist enum types into separate GameType structs
//TODO Define methods implementing each variant's game logic 
//TODO      Child process runs game loop 
pub enum Game {
    #[strum(serialize = "5CD")] FiveCardDraw(FiveCardDraw), 
    #[strum(serialize = "7CS")] SevenCardStud(SevenCardStud), 
    #[strum(serialize = "THE")] TexasHoldEm(TexasHoldEm)
}

#[derive(Debug, Clone)] 
pub struct FiveCardDraw {
    pub game_id Uuid, 
    pub table Table, 
    pub pot u32, 
    pub betting_rounds: BettingRounds, 
    pub betting_state: BettingState
}

impl FiveCardDraw {
    pub fn new() -> Self {
        let game_id = Uuid::new_v4(); 
        let mut deck = Deck::new();
        let mut pot = 0; 
        let mut table = Table::new(); 
        let mut betting_rounds = BettingRounds::PreDeal;  
        let mut betting_state = BettingState::new(); 
        let mut game_type = GameType::FiveCardDraw //default
    }

   

    //TODO implement other variant-specific methods 
}

impl SevenCardStud {
    pub fn new() -> Self {
        let game_id = Uuid::new_v4(); 
        let mut deck = Deck::new(); 
        let mut pot = 0; 
        let mut table = Table::new(); 
        let mut betting_rounds = BettingRounds::PreDeal; 
        let mut betting_state = BettingState::new(); 
    }

    //TODO implement other variant-specific methods
}

impl TexasHoldEm { 
    pub fn new() -> Self {
        let game_id = Uuid::new_v4(); 
        let mut deck = Deck::new(); 
        let mut pot = 0; 
        let mut table = Table::new(); 
        let mut betting_rounds = BettingRounds::PreDeal; 
        let mut betting_state = BettingState::new(); 
    }

    //TODO implement other variant-specific methods
}
