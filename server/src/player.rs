use uuid::Uuid;
use rocket::serde::{Serialize, Deserialize};
use poker_core::{Hand, Card};


// Player Statistics

/// Statistics tracked for each player.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PlayerStats {
    pub player_id: Uuid,
    pub username: String,
    pub rounds_played: u32,
    pub pots_won: u32,
    pub folds: u32,
    pub total_winnings: i64,
    pub total_losses: i64,
}

impl PlayerStats {
    pub fn new(player_id: Uuid, username: String) -> Self {
        Self {
            player_id,
            username,
            rounds_played: 0,
            pots_won: 0,
            folds: 0,
            total_winnings: 0,
            total_losses: 0,
        }
    }

    pub fn record_round_played(&mut self) {
        self.rounds_played += 1;
    }

    pub fn record_pot_won(&mut self, amount: u32) {
        self.pots_won += 1;
        self.total_winnings += amount as i64;
    }

    pub fn record_fold(&mut self) {
        self.folds += 1;
    }

    pub fn record_loss(&mut self, amount: u32) {
        self.total_losses += amount as i64;
    }
}

// Player Type

/// Represents a player at the poker table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub chips: u32,
    #[serde(skip, default)]
    pub hand: Hand,
    pub is_folded: bool,
    pub game_id: Option<Uuid>, 
    pub current_bet: u32,
}

impl Player {
    pub fn new(username: String, starting_chips: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            chips: starting_chips,
            hand: Hand::new(),
            game_id: Option<Uuid>, 
            is_folded: false,
            current_bet: 0,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    /// Discard cards at the given indices (descending, deduped), then add the new cards.
    /// Caller must deal exactly `unique_discard_count` cards and pass them here.
    pub fn draw(&mut self, discard_indices: &[usize], new_cards: Vec<Card>) -> Result<(), String> {
        for &idx in discard_indices {
            if idx >= self.hand.len() {
                return Err("Invalid card index".to_string());
            }
        }

        let mut sorted_indices = discard_indices.to_vec();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
        sorted_indices.dedup();

        if new_cards.len() != sorted_indices.len() {
            return Err("New cards count must match number of unique discard indices".to_string());
        }

        for &i in &sorted_indices {
            self.hand.remove_at(i);
        }

        for card in new_cards {
            self.hand.add(card);
        }

        Ok(())
    }
}

// Viewer Type 

/// Represents a viewer watching a game without participating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Viewer {
    pub id: Uuid,
    pub username: String,
    pub watching_game_id: Option<Uuid>,
}

impl Viewer {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            watching_game_id: None,
        }
    }
}
