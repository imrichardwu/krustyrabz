use strum_macros::Display;
use uuid::Uuid;

/// Stores round-types for all 3 poker variants.
/// PreDeal is shared across all 3 variants, it specifies that the game has not yet started.
#[derive(Debug, Clone, PartialEq, Copy, Display)]
pub enum BettingRound {
    #[strum(serialize = "predeal")]
    PreDeal,
    // Five Card Draw rounds
    #[strum(serialize = "predraw")]
    PreDraw,
    #[strum(serialize = "postdraw")]
    PostDraw,
    #[strum(serialize = "drawing")]
    Drawing,
    // Texas Hold'Em rounds
    #[strum(serialize = "preflop")]
    PreFlop,
    #[strum(serialize = "flop")]
    Flop,
    #[strum(serialize = "turn")]
    Turn,
    #[strum(serialize = "river")]
    River,
    // Seven Card Stud rounds
    #[strum(serialize = "third_street")]
    ThirdStreet,
    #[strum(serialize = "fourth_street")]
    FourthStreet,
    #[strum(serialize = "fifth_street")]
    FifthStreet,
    #[strum(serialize = "sixth_street")]
    SixthStreet,
    #[strum(serialize = "seventh_street")]
    SeventhStreet,
}

/// Represents an action taken by a player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BetAction {
    Check,
    Fold,
    Call,
    //Raise To (the total amount the player wishes to reach)
    RaiseTo(u32),
    AllIn,
}

/// Tracks the constraints and metadata of the current betting round.
#[derive(Debug, Clone)]
pub struct BettingState {
    pub to_call: u32,
    pub min_raise: u32,
    pub raises_used: u8,
    pub max_raises: u8,

    // Flow Control
    pub last_aggressor: Option<Uuid>,
}

impl BettingState {
    pub fn new() -> Self {
        Self {
            to_call: 0,
            min_raise: 10,
            raises_used: 0,
            max_raises: 3,
            last_aggressor: None,
        }
    }

    pub fn with_limits(min_raise: u32, max_raises: u8) -> Self {
        Self {
            to_call: 0,
            min_raise,
            raises_used: 0,
            max_raises,
            last_aggressor: None,
        }
    }

    pub fn reset_round(&mut self) {
        self.to_call = 0;
        self.raises_used = 0;
        self.last_aggressor = None;
    }
}

impl Default for BettingState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_betting_state_new_defaults() {
        let state = BettingState::new();

        assert_eq!(state.to_call, 0, "Initial amount to call should be 0");
        assert_eq!(state.min_raise, 10, "Default min raise should be 10");
        assert_eq!(state.raises_used, 0, "Initial raises used should be 0");
        assert_eq!(state.max_raises, 3, "Default max raises should be 3");
        assert_eq!(state.last_aggressor, None, "No aggressor initially");
    }

    #[test]
    fn test_betting_state_with_custom_limits() {
        let custom_min_raise = 50;
        let custom_max_raises = 5;
        let state = BettingState::with_limits(custom_min_raise, custom_max_raises);

        assert_eq!(state.to_call, 0);
        assert_eq!(
            state.min_raise, 50,
            "Min raise should match custom initialization"
        );
        assert_eq!(state.raises_used, 0);
        assert_eq!(
            state.max_raises, 5,
            "Max raises should match custom initialization"
        );
        assert_eq!(state.last_aggressor, None);
    }

    #[test]
    fn test_reset_round_clears_flow_state() {
        let mut state = BettingState::new();
        let dummy_player = Uuid::new_v4();

        // Simulate a round where someone raised
        state.to_call = 100;
        state.raises_used = 2;
        state.last_aggressor = Some(dummy_player);

        // Reset for the next street
        state.reset_round();

        // These should be cleared
        assert_eq!(state.to_call, 0, "to_call must reset to 0 for a new round");
        assert_eq!(
            state.raises_used, 0,
            "raises_used must reset to 0 for a new round"
        );
        assert_eq!(
            state.last_aggressor, None,
            "last_aggressor must be cleared for a new round"
        );

        // These underlying table limits should remain completely untouched
        assert_eq!(
            state.min_raise, 10,
            "Table minimum raise limit should persist across rounds"
        );
        assert_eq!(
            state.max_raises, 3,
            "Table maximum raise limit should persist across rounds"
        );
    }
}
