use thiserror::Error;

/// Errors produced by game logic and the house/matchmaking layer.
///
/// `Display` output is preserved from the previous `String`-based errors so that
/// HTTP response bodies (which embed these messages) remain byte-for-byte the same.
#[derive(Debug, Error)]
pub enum GameError {
    // --- Betting / turn errors (short codes consumed by the client) ---
    #[error("player_not_found")]
    PlayerNotFound,
    #[error("insufficient_funds")]
    InsufficientFunds,
    #[error("not_your_turn")]
    NotYourTurn,
    #[error("cannot_check_must_call")]
    CannotCheckMustCall,
    #[error("cannot_bet_must_raise")]
    CannotBetMustRaise,
    #[error("bet_too_small")]
    BetTooSmall,
    #[error("cannot_raise_must_bet")]
    CannotRaiseMustBet,
    #[error("raise_too_small")]
    RaiseTooSmall,
    #[error("invalid_action_for_betting_phase")]
    InvalidActionForBettingPhase,

    // --- Hand lifecycle ---
    #[error("hand_already_started")]
    HandAlreadyStarted,
    #[error("need_at_least_2_players")]
    NeedAtLeast2Players,
    #[error("need_at_least_2_active_players")]
    NeedAtLeast2ActivePlayers,
    #[error("not_enough_cards")]
    NotEnoughCards,
    #[error("no_active_players")]
    NoActivePlayers,
    #[error("no_up_cards_found")]
    NoUpCardsFound,
    #[error("game_not_started")]
    GameNotStarted,
    #[error("wrong_phase")]
    WrongPhase,
    #[error("wrong_phase_expecting_drawing")]
    WrongPhaseExpectingDrawing,

    // --- Pass / draw ---
    #[error("cannot_pass_first_round")]
    CannotPassFirstRound,
    #[error("already_drew_this_round")]
    AlreadyDrewThisRound,
    #[error("too_many_discards")]
    TooManyDiscards,

    // --- Draw-phase specific ---
    #[error("cannot_fold_in_draw_phase")]
    CannotFoldInDrawPhase,
    #[error("cannot_check_in_draw_phase")]
    CannotCheckInDrawPhase,
    #[error("cannot_call_in_draw_phase")]
    CannotCallInDrawPhase,
    #[error("cannot_bet_in_draw_phase")]
    CannotBetInDrawPhase,
    #[error("cannot_raise_in_draw_phase")]
    CannotRaiseInDrawPhase,
    #[error("cannot_pass_in_draw_phase")]
    CannotPassInDrawPhase,

    // --- Human-readable messages ---
    #[error("use per-action handlers for Five Card Draw")]
    UsePerActionHandlers,
    #[error("Sit out is only available for Seven Card Stud")]
    SitOutOnlySevenCardStud,
    #[error("Can only sit out before a hand starts")]
    SitOutBeforeHandStarts,
    #[error("Not in a betting phase")]
    NotInBettingPhase,
    #[error("Player not found at table")]
    PlayerNotFoundAtTable,
    #[error("only Five Card Draw supported")]
    OnlyFiveCardDrawSupported,
    #[error("AllIn not supported for Five Card Draw")]
    AllInNotSupportedFiveCardDraw,
    #[error("Invalid card index")]
    InvalidCardIndex,
    #[error("New cards count must match number of unique discard indices")]
    NewCardsCountMismatch,

    // --- Table operations (seating / removal) ---
    #[error("{0}")]
    Table(String),

    // --- House / matchmaking ---
    #[error("Cannot join a game that is currently in progress")]
    GameInProgress,
    #[error("Game not found: {0}")]
    GameNotFound(String),
    #[error("No available game found")]
    NoAvailableGame,
    #[error("Failed to remove game: {0}")]
    GameRemoveFailed(String),

    // --- Player loading (get_player_from_db) ---
    #[error("{0}")]
    InvalidUuid(String),
    #[error("Failed to create repository: {0}")]
    RepositoryCreate(String),
    #[error("Failed to get user: {0}")]
    UserLookup(String),
}
