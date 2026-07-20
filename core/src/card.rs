use strum_macros::{Display, EnumIter}; // strum for pretty printing

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub enum CardType {
    #[strum(to_string = "community")]
    Community,
    #[strum(to_string = "private")]
    Private,
    #[strum(to_string = "up")]
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Display)]
pub enum Suit {
    #[strum(to_string = "♣")]
    Clubs,
    #[strum(to_string = "♦")]
    Diamonds,
    #[strum(to_string = "♥")]
    Hearts,
    #[strum(to_string = "♠")]
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Display)]
pub enum Rank {
    #[strum(to_string = "2")]
    Two = 2,
    #[strum(to_string = "3")]
    Three, // 3
    #[strum(to_string = "4")]
    Four, // 4
    #[strum(to_string = "5")]
    Five, // 5
    #[strum(to_string = "6")]
    Six, // 6
    #[strum(to_string = "7")]
    Seven, // 7
    #[strum(to_string = "8")]
    Eight, // 8
    #[strum(to_string = "9")]
    Nine, // 9
    #[strum(to_string = "10")]
    Ten, // 10
    #[strum(to_string = "J")]
    Jack, // 11
    #[strum(to_string = "Q")]
    Queen, // 12
    #[strum(to_string = "K")]
    King, // 13
    #[strum(to_string = "A")]
    Ace, // 14
}

impl Rank {
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card {
    pub rank: Rank, // field_name : Type name (rank is type enum Rank)
    pub suit: Suit,
    pub card_type: CardType,
}

impl Card {
    pub fn construct(rank: Rank, suit: Suit, card_type: CardType) -> Self {
        Self {
            rank,
            suit,
            card_type,
        }
    }

    //Display implementation from strum can now be used directly here
    pub fn to_string(&self) -> String {
        format!("{}{}", self.rank, self.suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_construct_and_to_string() {
        let c = Card::construct(Rank::Ace, Suit::Spades, CardType::Private);
        assert_eq!(c.rank, Rank::Ace);
        assert_eq!(c.suit, Suit::Spades);
        assert_eq!(c.to_string(), "A♠");
    }

    #[test]
    fn rank_value() {
        assert_eq!(Rank::Two.value(), 2);
        assert_eq!(Rank::Ten.value(), 10);
        assert_eq!(Rank::Ace.value(), 14);
    }

    #[test]
    fn rank_ordering() {
        assert!(Rank::Ace > Rank::King);
        assert!(Rank::Two < Rank::Three);
    }

    #[test]
    fn suit_equality() {
        assert_eq!(Suit::Hearts, Suit::Hearts);
        assert_ne!(Suit::Clubs, Suit::Diamonds);
    }

    #[test]
    fn card_type_roundtrip() {
        let c = Card::construct(Rank::King, Suit::Hearts, CardType::Community);
        assert_eq!(c.card_type, CardType::Community);
    }
}
