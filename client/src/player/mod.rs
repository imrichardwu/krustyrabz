pub mod account;
pub mod statistics;

pub use account::PlayerAccount;
pub use statistics::PlayerStatistics;

/// Represents the current player in the game
pub struct Player {
    pub account: PlayerAccount,
    pub statistics: PlayerStatistics,
}

impl Player {
    /// Create a new player with the given username
    pub fn new(username: String) -> Self {
        Player {
            account: PlayerAccount::new(username),
            statistics: PlayerStatistics::new(),
        }
    }

    /// Get player's username
    pub fn get_username(&self) -> &str {
    }

    /// Get player's current chip balance
    pub fn get_chip_balance(&self) -> u64 {
    }

    /// Update chip balance (add or subtract)
    pub fn update_chips(&mut self, amount: i64) {
    }

    /// Set initial chip balance for a new game/session
    pub fn set_initial_chips(&mut self, chips: u64) {
    }
}
