/// Represents player statistics for tracking game performance
#[derive(Debug, Clone)]
pub struct PlayerStatistics {
    pub rounds_played: u32,
    pub games_won: u32,
    pub games_folded: u32,
    pub total_winnings: i64,
    pub total_losses: i64,
    pub biggest_win: u64,
    pub biggest_loss: u64,
}

impl PlayerStatistics {
    /// Create new player statistics
    pub fn new() -> Self {
        PlayerStatistics {
            rounds_played: 0,
            games_won: 0,
            games_folded: 0,
            total_winnings: 0,
            total_losses: 0,
            biggest_win: 0,
            biggest_loss: 0,
        }
    }

    /// Record a round played
    pub fn increment_rounds_played(&mut self) {
        self.rounds_played += 1;
    }

    /// Record a game won with winnings amount
    pub fn record_win(&mut self, winnings: u64) {
        self.games_won += 1;
        self.total_winnings += winnings as i64;
        if winnings > self.biggest_win {
            self.biggest_win = winnings;
        }
    }

    /// Record a game lost with loss amount
    pub fn record_loss(&mut self, loss: u64) {
        // Note: We track losses but not a separate games_lost count
        // If you need games_lost count, add it as a field to the struct
        self.total_losses += loss as i64;
        if loss > self.biggest_loss {
            self.biggest_loss = loss;
        }
    }

    /// Record a fold
    pub fn record_fold(&mut self) {
        self.games_folded += 1;
    }

    /// Calculate win rate (percentage of games won)
    pub fn get_win_rate(&self) -> f64 {
        if self.rounds_played == 0 {
            return 0.0;
        }
        ((self.games_won as f64) / (self.rounds_played as f64)) * 100.0
    }

    /// Calculate fold rate (percentage of games folded)
    pub fn get_fold_rate(&self) -> f64 {
        if self.rounds_played == 0 {
            return 0.0;
        }
        ((self.games_folded as f64) / (self.rounds_played as f64)) * 100.0
    }

    /// Calculate net profit/loss
    pub fn get_net_result(&self) -> i64 {
        self.total_winnings - self.total_losses
    }

    /// Get statistics summary as a formatted string
    pub fn get_summary(&self) -> String {
        format!(
            "=== Player Statistics ===\n\
             Rounds Played: {}\n\
             Games Won: {}\n\
             Games Folded: {}\n\
             Win Rate: {:.2}%\n\
             Fold Rate: {:.2}%\n\
             Total Winnings: {}\n\
             Total Losses: {}\n\
             Net Result: {}\n\
             Biggest Win: {}\n\
             Biggest Loss: {}",
            self.rounds_played,
            self.games_won,
            self.games_folded,
            self.get_win_rate(),
            self.get_fold_rate(),
            self.total_winnings,
            self.total_losses,
            self.get_net_result(),
            self.biggest_win,
            self.biggest_loss
        )
    }
}

impl Default for PlayerStatistics {
    fn default() -> Self {
        Self::new()
    }
}