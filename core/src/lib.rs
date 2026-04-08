#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Clubs,    
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    /// Returns a string representation of the suit
    fn to_string(&self) -> &str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank{
    Two = 2, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
    Jack, Queen, King, Ace,
}

impl Rank {

    /// Returns the display name of the rank
    fn to_string(&self) -> String {
        match self {
            Rank::Two => "2".to_string(),
            Rank::Three => "3".to_string(),
            Rank::Four => "4".to_string(),
            Rank::Five => "5".to_string(),
            Rank::Six => "6".to_string(),
            Rank::Seven => "7".to_string(),
            Rank::Eight => "8".to_string(),
            Rank::Nine => "9".to_string(),
            Rank::Ten => "10".to_string(),
            Rank::Jack => "J".to_string(),
            Rank::Queen => "Q".to_string(),
            Rank::King => "K".to_string(),
            Rank::Ace => "A".to_string(),
        }
    }

    /// Returns the numeric value (2–14)
    pub fn value(&self) -> u8 {
        *self as u8
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card {
    pub rank: Rank, // field_name : Type name (rank is type enum Rank)
    pub suit: Suit,
}

impl Card {

    /// Constructor for Card
    pub fn construct(rank: Rank, suit: Suit) -> Self { // No real constructors exist in rust, can stil define it as demonstrated below
        Self {rank, suit}
    }

    fn to_string(&self) -> String {
        format!("{}{}", self.rank.to_string(), self.suit.to_string())
    }
}