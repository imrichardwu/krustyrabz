use poker_core::{Card, CardType, DeckTrait, Rank, Suit};
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn construct() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::iter() {
            for rank in Rank::iter() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deck_has_52_cards() {
        let deck = Deck::construct();
        assert_eq!(deck.cards.len(), 52);
    }

    #[test]
    fn test_deck_deals_correctly() {
        let mut deck = Deck::construct();
        let initial_size = deck.cards.len();

        let cards = deck.deal(1);
        assert_eq!(cards.len(), 1);
        assert_eq!(deck.cards.len(), initial_size - 1);
    }

    #[test]
    fn test_empty_deck_returns_empty_vec() {
        let mut deck = Deck::construct();
        for _ in 0..52 {
            deck.deal(1);
        }

        assert!(deck.deal(1).is_empty());
    }

    #[test]
    fn test_deck_shuffle_changes_order() {
        let mut deck1 = Deck::construct();
        let deck2 = Deck::construct();

        deck1.shuffle();
        assert_ne!(deck1.cards, deck2.cards);
        assert_eq!(deck1.cards.len(), 52);
    }
}
