use crate::hand::Hand;
use uuid::Uuid;

pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub hand: Hand,
    pub chips: u32,
    pub current_bet: u32, // Tracked for betting logic
    pub is_folded: bool,  // Tracked for game logic
    pub game_id: Option<Uuid>,
}

impl Player {
    pub fn new(username: String, chips: u32, _game_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            hand: Hand::new(),
            chips,
            current_bet: 0,
            is_folded: false,
            game_id: None,
        }
    }
}
