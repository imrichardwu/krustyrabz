use crate::Card;

/// Trait for a deck that can deal cards
pub trait DeckTrait {
    fn deal(&mut self, count: usize) -> Vec<Card>;
}
