use crate::card::Card;
use crate::hand::DeckTrait;
use uuid::Uuid;

pub struct Player {
    pub id: Uuid,
    pub hand: Vec<Card>,
    pub chips: u32,
}

impl Player {
    pub fn new(chips: u32, id: Uuid) -> Self {
        Self {
            id,
            hand: Vec::new(),
            chips
        }
    }

    pub fn draw(&mut self, deck: &mut dyn DeckTrait, discard_indices: &[usize]) -> Result<(), String> {
        let mut sorted_indices = discard_indices.to_vec();
        sorted_indices.sort_by(|a, b| b.cmp(a));

        for &i in &sorted_indices {
            if i >= self.hand.len() {
                return Err("Invalid card index".to_string());
            }
            self.hand.remove(i);
        }

        let new_cards = deck.deal(discard_indices.len());
        self.hand.extend(new_cards);

        Ok(())
    }
}
