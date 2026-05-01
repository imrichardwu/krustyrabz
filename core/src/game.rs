use poker_core::{Deck,Player, BettingRounds, BettingState, Table, Card}; 
use strum_macros::EnumString; 
use std::sync::{Arc, Mutex}; 
use std::str::FromStr; 
use std::vec::vec; 
use uuid::Uuid; 

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

    //shuffle 
    //
    //deal 
    //
    //predraw_betting()
    //
    //draw 
    //
    //showdown 
    //
    //payout 
    //
    //reset 

    pub fn shuffle() -> Self {
        self.deck.shuffle() 
    }

    pub fn deal() -> Self {
        for deal in 1..=5 {
            for player in &mut Table.players {
                let mut card = self.deck.deal(1);
                card.card_type = Card::CardType::Private; 
                *player.hand.push(card); 
            }
        }
    }

    pub fn predraw_betting() -> Self {
    }

    pub fn draw() -> Self {
    }

    pub fn postdraw_betting() -> Self {
    }

    pub fn showdown() -> Self {
    }

    pub fn payout() -> Self {
    }

    pub fn reset() -> Self {
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

    pub fn shuffle() -> Self {
        ; 
    }

    pub fn deal() -> Self {
        ; 
    }

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
