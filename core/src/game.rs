use poker_core::{Deck,Player, BettingRounds, BettingState}; 
use strum_macros::EnumString; 
use std::sync::{Arc, Mutex}; 
use std::str::FromStr; 
use std::vec::vec; 
use uuid::Uuid; 

//TODO Hoist enum types into separate GameType structs
//TODO Define methods implementing each variant's game logic 
//TODO      Child process runs game loop 
pub enum GameType {
    #[strum(serialize = "5CD")] FiveCardDraw, 
    #[strum(serialize = "7CS")] SevenCardStud, 
    #[strum(serialize = "THE")] TexasHoldEm 
}

#[derive(Debug, Clone)] 
pub struct Game {
    pub game_id Uuid, 
    pub players vec<Player>, 
    pub pot_size u32, 
    pub betting_rounds: BettingRounds, 
    pub betting_state: BettingState, 
    pub game_type: GameType::FiveCardDraw //default
}

impl Game {
    pub fn new() -> Self {
        let game_id = Uuid::new_v4(); 
        let mut deck = Deck::new(); 
        let mut pot_size = 0; 
        let mut players = vec![]; 
        let mut betting_rounds = BettingRounds::PreDeal, 
        let mut betting_state = BettingState::new(), 
        let mut game_type = GameType::FiveCardDraw //default
    }

    pub fn get_player_count(&self) -> u32 {
       &self.players.len();  
    }

    pub fn seat_player_at_table(&self, &Player) -> Result<(), &'static str> {
        let count = &self.get_player_count(); 
        if (count < 5) {
            players.push(&Player); 
            Ok(()); 
        }
        else { 
            Err("table_full"); 
        }
    }
    
    pub fn get_game_type(&self) -> &GameType {
        &self.game_type;  
    }
    pub fn set_game_type(&mut self, game_type: GameType) {
        self.game_type = game_type;
    }
}
