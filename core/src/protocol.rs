// Shared Protocol Types for Client-Server Communication
//
// This module contains message structures used for HTTP communication
// between the poker client and server using Rocket/reqwest.

use serde::{Deserialize, Serialize};

// ============================================================================
// Request Types (Client -> Server)
// ============================================================================

/// Request to create a new game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub player_id: String,
    pub username: String,
    pub game_type: GameType,
}

/// Request to join an existing game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub player_id: String,
    pub username: String,
    pub game_id: String,
}

/// Request to perform a game action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub player_id: String,
    pub game_id: String,
    pub action: GameAction,
}

/// Request to register as a viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerRequest {
    pub viewer_id: String,
    pub game_id: String,
}

/// Request for player statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsRequest {
    pub player_id: String,
}

// ============================================================================
// Response Types (Server -> Client)
// ============================================================================

/// Generic server response for simple operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub success: bool,
    pub message: String,
}

impl ServerResponse {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// Response when creating or joining a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResponse {
    pub success: bool,
    pub message: String,
    pub game_id: Option<String>,
    pub game_state: Option<GameStateUpdate>,
}

impl GameResponse {
    pub fn success(message: impl Into<String>, game_id: String, state: GameStateUpdate) -> Self {
        Self {
            success: true,
            message: message.into(),
            game_id: Some(game_id),
            game_state: Some(state),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            game_id: None,
            game_state: None,
        }
    }
}

/// Response containing game state updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateUpdate {
    pub game_id: String,
    pub game_type: GameType,
    pub pot: u32,
    pub current_bet: u32,
    pub betting_round: BettingRound,
    pub action_on: Option<String>,
    pub player_count: usize,
    pub players: Vec<PlayerInfo>,
    pub community_cards: Vec<String>,
    pub your_hand: Vec<String>,
    pub your_chips: u32,
}

/// Information about a player visible to others.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub username: String,
    pub chips: u32,
    pub current_bet: u32,
    pub folded: bool,
    pub is_dealer: bool,
    pub cards_count: usize, // Number of cards (actual cards hidden)
}

/// Player statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub player_id: String,
    pub username: String,
    pub rounds_played: u32,
    pub pots_won: u32,
    pub folds: u32,
    pub total_winnings: i64,
    pub current_balance: u32,
}

/// List of available games.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameListResponse {
    pub games: Vec<GameSummary>,
}

/// Summary of a game for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSummary {
    pub game_id: String,
    pub game_type: GameType,
    pub player_count: usize,
    pub max_players: usize,
    pub status: GameStatus,
    pub pot: u32,
}

/// House rules response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseRules {
    pub min_bet: u32,
    pub max_bet: u32,
    pub max_raises_per_round: u32,
    pub starting_chips: u32,
    pub ante: u32,
    pub small_blind: u32,
    pub big_blind: u32,
}

// ============================================================================
// Enums
// ============================================================================

/// Types of poker games supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameType {
    FiveCardDraw,
    SevenCardStud,
    TexasHoldEm,
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::FiveCardDraw => write!(f, "Five Card Draw"),
            GameType::SevenCardStud => write!(f, "Seven Card Stud"),
            GameType::TexasHoldEm => write!(f, "Texas Hold'em"),
        }
    }
}

/// Status of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    WaitingForPlayers,
    InProgress,
    Finished,
}

/// Betting rounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BettingRound {
    PreDraw,    // Five Card Draw
    PostDraw,   // Five Card Draw
    ThirdStreet,  // Seven Card Stud
    FourthStreet, // Seven Card Stud
    FifthStreet,  // Seven Card Stud
    SixthStreet,  // Seven Card Stud
    River,        // Seven Card Stud / Texas Hold'em
    PreFlop,      // Texas Hold'em
    Flop,         // Texas Hold'em
    Turn,         // Texas Hold'em
    Showdown,
}

impl std::fmt::Display for BettingRound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BettingRound::PreDraw => write!(f, "Pre-Draw"),
            BettingRound::PostDraw => write!(f, "Post-Draw"),
            BettingRound::ThirdStreet => write!(f, "Third Street"),
            BettingRound::FourthStreet => write!(f, "Fourth Street"),
            BettingRound::FifthStreet => write!(f, "Fifth Street"),
            BettingRound::SixthStreet => write!(f, "Sixth Street"),
            BettingRound::River => write!(f, "River"),
            BettingRound::PreFlop => write!(f, "Pre-Flop"),
            BettingRound::Flop => write!(f, "Flop"),
            BettingRound::Turn => write!(f, "Turn"),
            BettingRound::Showdown => write!(f, "Showdown"),
        }
    }
}

/// Types of actions a player can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    Fold,
    Check,
    Call,
    Bet { amount: u32 },
    Raise { amount: u32 },
    AllIn,
    Draw { discard_indices: Vec<usize> }, // For Five Card Draw
}

impl std::fmt::Display for GameAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameAction::Fold => write!(f, "Fold"),
            GameAction::Check => write!(f, "Check"),
            GameAction::Call => write!(f, "Call"),
            GameAction::Bet { amount } => write!(f, "Bet ${}", amount),
            GameAction::Raise { amount } => write!(f, "Raise ${}", amount),
            GameAction::AllIn => write!(f, "All In"),
            GameAction::Draw { discard_indices } => {
                write!(f, "Draw {} cards", discard_indices.len())
            }
        }
    }
}
