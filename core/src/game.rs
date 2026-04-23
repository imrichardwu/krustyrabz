use poker_core::{Card, Rank, Suit, DeckTrait, Player}; 
use strum_macros::EnumString; 
use std::str::FromStr; 
use std::vec::vec; 
use uuid::Uuid; 

#[derive(Debug)] 
pub enum GameType {
    #[strum(serialize = "5CD")] FiveCardDraw, 
    #[strum(serialize = "7CS")] SevenCardStud, 
    #[strum(serialize = "THE")] TexasHoldEm 
}

pub struct Game {
    pub game_id Uuid, 
    pub players vec<Player>,
    pub pot_size u32, 
    pub pi
}

impl Game {
    pub fn construct() -> Self {
        let mut game_id = Uuid::new_v4(); 
    }
    pub fn get_player_count(); 
}
