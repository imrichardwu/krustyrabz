use crate::card::Card;

pub struct Player {
    pub hand: Vec<Card>,
    pub chips: u32,
}

impl Player {
    pub fn new(chips: u32) -> Self {
        Self {
            hand: Vec::new(),
            chips
        }
    }

    pub fn draw(&mut self, deck: &mut crate::hand::Deck, discard_indices: &[usize]) -> Result<(), String> {
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
