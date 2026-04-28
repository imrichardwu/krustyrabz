use std::time::SystemTime;

/// Represents a player's account information
#[derive(Debug, Clone)]
pub struct PlayerAccount {
    pub username: String,
    pub email: Option<String>,
    pub chip_balance: u64,
    pub account_created: SystemTime,
    pub last_login: Option<SystemTime>,
}

impl PlayerAccount {
    /// Create a new player account
    pub fn new(username: String) -> Self {
        PlayerAccount {
            username,
            email: None,
            chip_balance: 0,
            account_created: SystemTime::now(),
            last_login: None,
        }
    }

    /// Set the player's email
    pub fn set_email(&mut self, email: String) {
        self.email = Some(email);
    }

    /// Get the player's email
    pub fn get_email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Deposit chips into account
    pub fn deposit_chips(&mut self, amount: u64) {
        self.chip_balance += amount;
    }

    /// Withdraw chips from account (returns true if successful)
    pub fn withdraw_chips(&mut self, amount: u64) -> bool {
        if amount > self.chip_balance {
            return false;
        }
        self.chip_balance -= amount;
        true
    }

    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login = Some(SystemTime::now());
    }

    /// Get account balance summary as a string
    pub fn get_balance_summary(&self) -> String {
        format!(
            "=== Account Balance ===\n\
             Username: {}\n\
             Email: {}\n\
             Chip Balance: {}\n\
             Account Created: {}\n\
             Last Login: {}",
            self.username,
            self.email.as_deref().unwrap_or("N/A"),
            self.chip_balance,
            self.account_created.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            self.last_login.unwrap_or(SystemTime::UNIX_EPOCH).duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
        )
    }
}
