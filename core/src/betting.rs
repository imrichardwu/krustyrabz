
#[derive(Debug, Clone)]pub enum BetAction {
    Check,
    Call,
    Bet { amount: u32 },
    Raise { amount: u32 },
    Fold,
}

pub enum BettingOutcome {
    MatchedHighestBet, 
    Folded
    //Continue, // next player acts
    //BettingRoundComplete,
    //HandComplete { winner: Uuid },
}
