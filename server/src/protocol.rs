// Handles Client Message Protocol
//
// This module contains message structures for client-server communication.

use rocket::serde::{Serialize, Deserialize};

// Client Message Protocol

/// This struct stores a serialized client message struct which contains 
/// actions to execute on behalf of clients as well as action data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub payload: String,
}

impl Message {
    pub fn new(payload: String) -> Self {
        Self { payload }
    }
}

/// Server response to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ServerResponse {
    pub success: bool,
    pub message: String,
    pub game_info: Option<String>,
}

impl ServerResponse {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            game_info: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            game_info: None,
        }
    }

    pub fn with_game_info(mut self, info: String) -> Self {
        self.game_info = Some(info);
        self
    }
}

// Game Action Messages (for future expansion)

/// Types of actions a player can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum GameAction {
    Fold,
    Check,
    Call,
    Bet { amount: u32 },
    Raise { amount: u32 },
    Draw { discard_indices: Vec<usize> },
}

/// A request from the client to perform an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ActionRequest {
    pub player_id: String,
    pub game_id: String,
    pub action: GameAction,
}

/// Response containing game state updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GameStateUpdate {
    pub game_id: String,
    pub pot: u32,
    pub current_bet: u32,
    pub betting_round: String,
    pub action_on: Option<String>,
    pub player_count: usize,
}
