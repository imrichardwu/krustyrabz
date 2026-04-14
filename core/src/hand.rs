use std::cmp::Ordering;
use crate::card::{Card, Rank, Suit};
use arrayvec::ArrayVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandCategory {
    HighCard, // Lowest
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush, // Highest
}

#[derive(Debug, Eq, PartialEq)]
pub struct HandRank {
    pub category: HandCategory,
    pub kickers: Vec<u8>,
}

impl Ord for HandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        // Switch-Case on the condition if HandCategories are equal then compare the kickers
        match self.category.cmp(&other.category) {
            Ordering::Equal => self.kickers.cmp(&other.kickers),
            other => other
        }
    }
}

// Had to define this cuz rust complains otherwise if only Ord is defined.
impl PartialOrd for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
// using ArrayVec for stack allocation, vector-like properties since Hand size is capped at 5
pub struct Hand {
    cards: ArrayVec<Card, 5>, 
}

impl Hand {
    /// Create an empty hand (for dealing)
    pub fn new() -> Self {
        Self {
            cards: ArrayVec::new(),
        }
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn analyze_hand(&self) -> ([u8; 13], bool) {
        let mut counts = [0u8; 13];
        
        if self.cards.is_empty() {
            return (counts, false);
        }

        let mut is_flush = true;
        let first_suit = self.cards[0].suit;

        // Note: Rank::Two = 2, so index = rank as usize - 2
        for elem in &self.cards {
            counts[elem.rank as usize - 2] += 1;
            if elem.suit != first_suit {
                is_flush = false;
            }
        }
        (counts, is_flush)
    }

    pub fn evaluate(&self) -> HandRank {
        let (counts, is_flush) = self.analyze_hand();
        todo!()
    }
}

/// Trait for a deck that can deal cards
pub trait DeckTrait {
    fn deal(&mut self, count: usize) -> Vec<Card>;
}
