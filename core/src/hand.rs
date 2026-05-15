use crate::card::{Card, Rank, Suit};
use arrayvec::ArrayVec;
use std::cmp::Ordering;

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
    pub kickers: ArrayVec<Rank, 5>,
}

impl Ord for HandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        // Switch-Case on the condition if HandCategories are equal then compare the kickers
        match self.category.cmp(&other.category) {
            Ordering::Equal => self.kickers.cmp(&other.kickers),
            other => other,
        }
    }
}

// Had to define this cuz rust complains otherwise if only Ord is defined.
impl PartialOrd for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Hand {
    // Increased to 7 to support SevenCardStud and Hold'em (best 5 of 7)
    // For 5-Card Draw, we just use the first 5 slots.
    cards: ArrayVec<Card, 7>,
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

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Clears the hand
    pub fn clear(&mut self) {
        self.cards.clear();
    }

    /// Expose cards slice (useful for UI/Rendering)
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Internal helper to get rank frequencies and flush status.
    /// Works for both 5-card and 7-card hands.
    fn analyze_hand(&self, cards: &[Card]) -> ([u8; 13], bool, u16) {
        let mut counts = [0u8; 13];

        if self.cards.is_empty() {
            return (counts, false, 0);
        }

        let mut is_flush = true;
        let first_suit = self.cards[0].suit;
        let mut rank_mask = 0u16;

        // Note: Rank::Two = 2, so index = rank as usize - 2
        for elem in &self.cards {
            let index = elem.rank as usize - 2;
            counts[index] += 1;
            rank_mask |= 1 << index; // build bitmask
            if elem.suit != first_suit {
                is_flush = false;
            }
        }

        (counts, is_flush, rank_mask)
    }

    pub fn evaluate(&self) -> HandRank {
        match self.len() {
            5 => self.evaluate_5cards(self.cards.as_slice()),

            7 => {
                let mut best_rank: Option<HandRank> = None;

                // 7C5 is the same as 7C2 that is choose any 2 cards from 7 to remove
                for i in 0..7 {
                    for j in (i + 1)..7 {
                        // Create a temporary stack buffer for the 5 cards
                        let mut buffer = ArrayVec::<Card, 5>::new();

                        for (index, &card) in self.cards.iter().enumerate() {
                            if index != i && index != j {
                                buffer.push(card);
                            }
                        }

                        // buffer is ArrayVec<Card, 5>, which derefs to &[Card]
                        let rank = self.evaluate_5cards(&buffer);

                        match best_rank {
                            None => best_rank = Some(rank),
                            Some(ref current) => {
                                if rank > *current {
                                    best_rank = Some(rank);
                                }
                            }
                        }
                    }
                }

                best_rank.expect("7-card evaluation failed to find any combinations")
            }

            _ => panic!(
                "Invalid hand size: {}. Only 5 or 7 cards supported.",
                self.len()
            ),
        }
    }

    fn evaluate_5cards(&self, cards: &[Card]) -> HandRank {

        let (counts, is_flush, mask) = self.analyze_hand(cards);
        let straight_high_card = Self::check_straight_mask(mask);
        let is_straight = straight_high_card.is_some();

        let mut category = HandCategory::HighCard;

        if is_flush && is_straight {

            if straight_high_card == Some(14) {
                category = HandCategory::RoyalFlush;

            } else {
                category = HandCategory::StraightFlush;
            }
            
        } else if counts.contains(&4) {
            category = HandCategory::FourOfAKind;

        } else if counts.contains(&3) && counts.contains(&2) {
            category = HandCategory::FullHouse;

        } else if is_flush {
            category = HandCategory::Flush;

        } else if is_straight {
            category = HandCategory::Straight;

        } else if counts.contains(&3) {
            category = HandCategory::ThreeOfAKind;

        } else {
            let pairs = counts.iter().filter(|&&c| c == 2).count();
            if pairs == 2 {
                category = HandCategory::TwoPair;
            } else if pairs == 1 {
                category = HandCategory::Pair;
            }
        }

        let mut kickers = ArrayVec::new();

        match category {
            // the kicker is just the top card of the straight here
            HandCategory::Straight | HandCategory::StraightFlush => {
                let rank_val = straight_high_card.unwrap();
                kickers.push(Self::rank_from_u8(rank_val));
            }
            HandCategory::RoyalFlush => {
                // No kickers needed
            }
            _ => {
                // Pass 1: Add the cards defining the hand
                match category {
                    HandCategory::FourOfAKind => self.push_ranks_by_count(&mut kickers, &counts, 4),
                    HandCategory::FullHouse => {
                        self.push_ranks_by_count(&mut kickers, &counts, 3);
                        self.push_ranks_by_count(&mut kickers, &counts, 2);
                    }
                    HandCategory::ThreeOfAKind => {
                        self.push_ranks_by_count(&mut kickers, &counts, 3)
                    }
                    HandCategory::TwoPair | HandCategory::Pair => {
                        self.push_ranks_by_count(&mut kickers, &counts, 2);
                    }
                    _ => {}
                }

                // Pass 2: Add the leftovers
                self.push_ranks_by_count(&mut kickers, &counts, 1);
            }
        }

        HandRank { category, kickers }
    }

    fn check_straight_mask(mask: u16) -> Option<u8> {
        let mut i = 12;
        while i >= 4 {
            if (mask >> (i - 4)) & 0b11111 == 0b11111 {
                return Some((i + 2) as u8);
            }
            i -= 1;
        }
        if mask & 0b1000000001111 == 0b1000000001111 {
            return Some(5);
        }
        None
    }

    fn push_ranks_by_count(&self, kickers: &mut ArrayVec<Rank, 5>, counts: &[u8; 13], target_count: u8) {
        // Iterate backwards from index 12 (Ace) to 0 (Two)
        for i in (0..13).rev() {
            if counts[i] == target_count {
                let rank_val = (i + 2) as u8;
                kickers.push(Self::rank_from_u8(rank_val));
            }
        }
    }

    fn rank_from_u8(v: u8) -> Rank {
        match v {
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            12 => Rank::Queen,
            13 => Rank::King,
            14 => Rank::Ace,
            _ => panic!("Invalid rank value: {}", v),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardType;

    fn card(r: Rank, s: Suit) -> Card {
        Card::construct(r, s, CardType::Private)
    }

    #[test]
    fn hand_new_empty() {
        let h = Hand::new();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn hand_add_and_len() {
        let mut h = Hand::new();
        h.add(card(Rank::Ace, Suit::Spades));
        h.add(card(Rank::King, Suit::Spades));
        assert_eq!(h.len(), 2);
        assert!(!h.is_empty());
        assert_eq!(h.cards().len(), 2);
    }

    #[test]
    fn hand_clear() {
        let mut h = Hand::new();
        h.add(card(Rank::Two, Suit::Hearts));
        h.clear();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn evaluate_high_card() {
        let mut h = Hand::new();
        h.add(card(Rank::Ace, Suit::Spades));
        h.add(card(Rank::King, Suit::Hearts));
        h.add(card(Rank::Ten, Suit::Clubs));
        h.add(card(Rank::Five, Suit::Diamonds));
        h.add(card(Rank::Two, Suit::Spades));
        let rank = h.evaluate();
        assert_eq!(rank.category, HandCategory::HighCard);
    }

    #[test]
    fn evaluate_pair() {
        let mut h = Hand::new();
        h.add(card(Rank::Ace, Suit::Spades));
        h.add(card(Rank::Ace, Suit::Hearts));
        h.add(card(Rank::King, Suit::Clubs));
        h.add(card(Rank::Ten, Suit::Diamonds));
        h.add(card(Rank::Two, Suit::Spades));
        let rank = h.evaluate();
        assert_eq!(rank.category, HandCategory::Pair);
    }

    #[test]
    fn evaluate_flush() {
        let mut h = Hand::new();
        for r in [Rank::Ace, Rank::King, Rank::Ten, Rank::Five, Rank::Two] {
            h.add(card(r, Suit::Spades));
        }
        let rank = h.evaluate();
        assert_eq!(rank.category, HandCategory::Flush);
    }

    #[test]
    fn evaluate_straight() {
        let mut h = Hand::new();
        h.add(card(Rank::Nine, Suit::Spades));
        h.add(card(Rank::Eight, Suit::Hearts));
        h.add(card(Rank::Seven, Suit::Clubs));
        h.add(card(Rank::Six, Suit::Diamonds));
        h.add(card(Rank::Five, Suit::Spades));
        let rank = h.evaluate();
        assert_eq!(rank.category, HandCategory::Straight);
    }

    #[test]
    fn evaluate_royal_flush() {
        let mut h = Hand::new();
        for r in [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten] {
            h.add(card(r, Suit::Hearts));
        }
        let rank = h.evaluate();
        assert_eq!(rank.category, HandCategory::RoyalFlush);
    }

    #[test]
    fn hand_rank_ordering() {
        let mut high = Hand::new();
        high.add(card(Rank::Ace, Suit::Spades));
        high.add(card(Rank::King, Suit::Hearts));
        high.add(card(Rank::Ten, Suit::Clubs));
        high.add(card(Rank::Five, Suit::Diamonds));
        high.add(card(Rank::Two, Suit::Spades));
        let mut pair = Hand::new();
        pair.add(card(Rank::Ace, Suit::Spades));
        pair.add(card(Rank::Ace, Suit::Hearts));
        pair.add(card(Rank::King, Suit::Clubs));
        pair.add(card(Rank::Ten, Suit::Diamonds));
        pair.add(card(Rank::Two, Suit::Spades));
        assert_eq!(high.evaluate().category, HandCategory::HighCard);
        assert_eq!(pair.evaluate().category, HandCategory::Pair);
        assert!(pair.evaluate() > high.evaluate());
    }
}

/// Trait for a deck that can deal cards
pub trait DeckTrait {
    fn deal(&mut self, count: usize) -> Vec<Card>;
}
