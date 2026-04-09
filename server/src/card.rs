use poker_core::{Card, Rank, Suit};
use Suit::*;
use Rank::*;
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn standard() -> Self {
        

        let suits = [Clubs, Diamonds, Hearts, Spades];
        let ranks = [Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King];

        let mut cards = Vec::new();

        for suit in suits {
            for rank in ranks {
                cards.push(Card {suit, rank});
            }
        }

        Self { cards }
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn deal(&mut self, count: usize) -> Vec<Card> {
        let min_draw = count.min(self.cards.len());
        self.cards.drain(..min_draw).collect() 
    }
}