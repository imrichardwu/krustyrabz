mod card;
use card::{Deck};

fn main() {
    let deck = Deck::standard();
    println!("Deck has {} cards", deck.cards.len());
    println!("First: {:?}, last: {:?}", deck.cards[0], deck.cards[51]);
}
