use crate::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rand::rng;
use strum::IntoEnumIterator; // using strum to expose an iterator on the enum for easier access

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn construct() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::iter(){
            for rank in Rank::iter(){
                cards.push(Card::construct(rank, suit))
            }
        }
        Self { cards }
    }

    /// Shuffle using Fisher-Yates
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::rng());
    } 

    /// Dealing, return type set to Option since may not always have cards to deal
    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Remaining cards in deck
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    
    /// Bool flag to check if deck is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}