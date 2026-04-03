#[derive(Debug, Copy, Clone)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Copy, Clone)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug)]
pub struct Deck {
    pub cards: [Card; 52],
}

impl Deck {
    pub fn standard() -> Self {
        use Suit::*;
        use Rank::*;

        let suits = [Clubs, Diamonds, Hearts, Spades];
        let ranks = [Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King];

        let mut cards = Vec::new();

        for suit in suits {
            for rank in ranks {
                cards.push(Card {suit, rank});
            }
        }

        Self { cards: cards.try_into().unwrap() }
    }

    pub fn shuffle(&mut self)  {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn deal(&mut self, count: usize) -> Vec<Card> {
        let dealt_cards = self.cards[..count].to_vec();
        self.cards = self.cards[count..].to_vec().try_into().unwrap();
        dealt_cards
    }
}