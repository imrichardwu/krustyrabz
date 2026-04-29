use serde::{Deserialize, Serialize};

/// Represents all message types between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    // Authentication messages
    RegisterRequest,
    RegisterResponse,
    LoginRequest,
    LoginResponse,
    LogoutRequest,

    // Game messages
    JoinTableRequest,
    JoinTableResponse,
    LeaveTableRequest,
    LeaveTableResponse,

    // Betting messages
    BetAction,
    FoldAction,
    CheckAction,
    CallAction,
    RaiseAction,
    AllInAction,

    // Game state messages
    DealerUpdate,
    CommunityCardsUpdate,
    PlayerHandUpdate,
    PotUpdate,
    BettingRoundUpdate,
    GameResultUpdate,

    // Viewer messages
    JoinAsViewerRequest,
    JoinAsViewerResponse,

    // Statistics messages
    StatisticsRequest,
    StatisticsResponse,

    // Error message
    Error,

    // Ping/Pong for keep-alive
    Ping,
    Pong,
}

/// Generic message structure for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub sender_id: String,
    pub receiver_id: Option<String>,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(
        message_type: MessageType,
        sender_id: String,
        payload: Vec<u8>,
    ) -> Self {

    }

    /// Set receiver ID
    pub fn with_receiver(mut self, receiver_id: String) -> Self {

    }

    /// Serialize message to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {

    }

    /// Deserialize message from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {

    }

    /// Get message timestamp
    pub fn get_timestamp(&self) -> u64 {

    }

    /// Get message sender
    pub fn get_sender(&self) -> &str {

    }
}

// Message builders for common message types

/// Build a login request message
pub fn build_login_request(username: String, password: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
}

/// Build a register request message
pub fn build_register_request(
    username: String,
    email: String,
    password: String,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
}

/// Build a join table request message
pub fn build_join_table_request(table_id: u32, buy_in: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
}

/// Build a bet action message
pub fn build_bet_action(amount: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
}

/// Build a fold action message
pub fn build_fold_action() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
}