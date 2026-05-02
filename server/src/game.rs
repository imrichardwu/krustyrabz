use poker_core::{Deck,Player, Table, Card}; 
use crate::betting::{BettingRounds, BettingState};
use strum_macros::EnumString; 
use std::sync::{Arc, Mutex}; 
use std::str::FromStr; 
use std::vec::vec; 
use uuid::Uuid; 

//This enum is a "generic variant" for populating the House's "live_games" vector, 
//which is a heterogeneous data structure containing live or pending games of poker. 
//Each game can be one of three different variants: FiveCardDraw, SevenCardStud, 
//or TexasHoldEm. 
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

    //Round - Step 1
    pub fn shuffle() -> Result<(), 'static String> {
        self.deck.shuffle() 
    }

    //Round - Step 2 
    //Each player is dealt five private cards which form their hand. 
    pub fn deal() -> Result<(), 'static String> {
        for deal in 1..=5 {
            for mut player in &mut self.table.players {
                //on the very first deal, every player's hand should be empty 
                if player.hand.len() && deal == 1 {
                    return Err(format!("No cards have been dealt, but {} somehow has {} card(s)
                                        in hand. This should be impossible.", player.id, 
                                        *player.hand.cards.len()))
                }
                let mut card = self.deck.deal(1);
                card.card_type = Card::CardType::Private; 
                *player.hand.add(card); 
                Ok(()); 
            }
        }
    }

    //Round - Step 3 
    //This is the first betting round. It begins with the player to the dealer's left, 
    //which for simplicity is defined as the Player located at index 0 in the Table's 
    //Player Vec. 
    pub fn predraw_betting() -> Result<(), 'static String> {
        for &player in &self.table.players { 
        }
    }

    //Round - Step 4 
    pub fn draw() -> () {
    }

    pub fn postdraw_betting() -> () {
    }

    pub fn showdown() -> () {
    }

    pub fn payout() -> () {
    }

    pub fn reset() -> () {
    }

    //TODO implement other variant-specific methods 
}

/*
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
*/
