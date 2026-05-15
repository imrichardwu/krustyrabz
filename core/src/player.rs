//use crate::card::Card;
use crate::hand::DeckTrait;
use crate::hand::Hand; // Using my implementation of Hand in place of a vector
use uuid::Uuid;

pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub hand: Hand,
    pub chips: u32,
    pub current_bet: u32, // Tracked for betting logic
    pub is_folded: bool,  // Tracked for game logic
    pub game_id: Uuid,
}

impl Player {
    pub fn new(username: String, chips: u32, game_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            hand: Hand::new(),
            chips,
            current_bet: 0,
            is_folded: false,
            game_id,
        }
    }

    // draw function that works with ArrayVec
    pub fn draw(&mut self, deck: &mut dyn DeckTrait, discard_indices: &[usize]) -> Result<(), String> {

        for &idx in discard_indices {
            if idx >= self.hand.len() {
                return Err("Invalid card index".to_string());
            }
        }

        let mut sorted_indices = discard_indices.to_vec();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
        sorted_indices.dedup();

        
        for &i in &sorted_indices {
             self.hand.remove_at(i); 
        }

        let new_cards = deck.deal(sorted_indices.len());
        for card in new_cards {
            self.hand.add(card);
        }

        Ok(())
    }

    pub fn get_action(&self) -> () {
        todo!()
    }
}
