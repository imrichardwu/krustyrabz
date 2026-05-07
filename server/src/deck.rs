use poker_core::{Card, Rank, Suit, CardType, DeckTrait};
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn construct() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::iter(){
            for rank in Rank::iter(){
                cards.push(Card::construct(rank, suit, CardType::Community))
            }
        }
        Self { cards }
    }

    pub fn standard() -> Self {
        Self::construct()
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::rng());
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl DeckTrait for Deck {
    fn deal(&mut self, count: usize) -> Vec<Card> {
        let min_draw = count.min(self.cards.len());
        self.cards.drain(..min_draw).collect() 
    }
}