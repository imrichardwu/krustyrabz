use poker_core::{Card, Rank, Suit};
use Suit::*;
use Rank::*;
use rand::seq::SliceRandom;
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
        self.cards.shuffle(&mut rand::rng());
    }

    pub fn deal(&mut self, count: usize) -> Vec<Card> {
        let min_draw = count.min(self.cards.len());
        self.cards.drain(..min_draw).collect() 
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}