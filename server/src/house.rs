// House Module - Poker Game Server
//
// This module contains the House struct which manages all live games,
// and provides HTTP route handlers for the Rocket web framework.

use uuid::Uuid;
use crate::game::Game;
use poker_core::{
    ActionRequest, AddChipsRequest, AddChipsResponse, CreateGameRequest,
    GameListResponse, GameResponse, GameStateUpdate, GameSummary, GameType, 
    HouseRules, JoinGameRequest, PlayerInfo, PlayerStats, ServerResponse,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rocket::{get, post, State};
use rocket::serde::json::Json;

use storage::DatabaseConnection; 

use crate::game::{FiveCardDraw, SevenCardStud, TexasHoldEm};
use crate::player::Player;

// ============================================================================
// House Structure
// ============================================================================

/// This data structure stores all currently live and pending games.
/// Games are stored in a HashMap keyed by game_id (as String for protocol compatibility).
#[derive(Debug)]
pub struct House {
    pub live_games: Arc<Mutex<HashMap<String, Game>>>,
}

impl House {
    pub fn new() -> Self {
        Self {
            live_games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // ========================================================================
    // Game Management Helper Methods
    // ========================================================================

    /// Helper method to create a new game of the specified type.
    ///
    /// Parameters:
    ///     game_type - The type of game to create (FiveCardDraw, SevenCardStud, TexasHoldEm)
    ///
    /// Returns:
    ///     Result<Game, String> - The created game or an error message
    pub fn create_new_game(&self, game_type: GameType) -> Result<Game, String> {
        match game_type {
            GameType::FiveCardDraw => {
                let game = FiveCardDraw::new();
                Ok(Game::FiveCardDraw(game))
            }
            GameType::SevenCardStud => {
                let game = SevenCardStud::new();
                Ok(Game::SevenCardStud(game))
            }
            GameType::TexasHoldEm => {
                let game = TexasHoldEm::new();
                Ok(Game::TexasHoldEm(game))
            }
        }
    }

    /// Helper method to lock the live_games HashMap and attempt to add a player
    /// to a specific game or find an open game.
    ///
    /// If game_id is provided, attempts to add the player to that specific game.
    /// If game_id is None, searches for the first available game of the requested type.
    ///
    /// Parameters:
    ///     player    - The player to add
    ///     game_type - Optional game type to search for
    ///     game_id   - Optional specific game ID to join
    ///
    /// Returns:
    ///     Result<String, String> - The game_id on success, or an error message
    pub fn lock_and_add(
        &self,
        player: Player,
        game_type: Option<GameType>,
        game_id: Option<String>,
    ) -> Result<String, String> {
        let mut games = self.live_games.lock().unwrap();

        match game_id {
            Some(id) => {
                // Try to join specific game
                match games.get_mut(&id) {
                    Some(game) => {
                        game.add_player(player)?;
                        Ok(id)
                    }
                    None => Err(format!("Game not found: {}", id)),
                }
            }
            None => {
                // Find first available game of requested type
                if let Some(requested_type) = game_type {
                    for (id, game) in games.iter_mut() {
                        if game.get_game_type() == requested_type && !game.is_full() {
                            match game.add_player(player.clone()) {
                                Ok(_) => return Ok(id.clone()),
                                Err(_) => continue,
                            }
                        }
                    }
                }
                Err("No available game found".to_string())
            }
        }
    }

    pub fn remove_game(&self, game_id: &str) -> Result<(), String> { 
        let mut games = self.live_games.lock().unwrap(); 
        games.remove(game_id).ok_or(format!("Failed to remove game: {}", game_id))?;
        Ok(())
    }

    /// Helper method to remove a player from a game when they disconnect.
    ///
    /// Parameters:
    ///     game_id   - The game to remove the player from
    ///     player_id - The player to remove
    ///
    /// Returns:
    ///     Result<(), String>
    pub fn remove_player(&self, game_id: &str, player_id: Uuid) -> Result<(), String> {
        let mut games = self.live_games.lock().unwrap();
        match games.get_mut(game_id) {
            Some(game) => {
                game.remove_player(player_id)?;
                // TODO: If game is empty, consider removing it from live_games
                if game.is_empty() { self.remove_game(game_id); }
                Ok(()) 
            },
            None => Err(format!("Game not found: {}", game_id).to_string()),
        }
    }

    /// Attempts to find an open game for the player. If no open game is found,
    /// creates a new one and adds the player to it.
    ///
    /// Parameters:
    ///     player    - The player to seat at a table
    ///     game_type - The type of game requested
    ///
    /// Returns:
    ///     Result<String, String> - The game_id on success
    pub fn find_or_create_game(&self, player: Player, game_type: GameType) -> Result<String, String> {
        // Try to find an existing game
        let result = self.lock_and_add(player.clone(), Some(game_type), None);

        match result {
            Ok(game_id) => Ok(game_id),
            Err(_) => {
                // No game found, create a new one
                let mut new_game = self.create_new_game(game_type)?;
                let game_id = new_game.get_game_id().to_string();
                new_game.add_player(player)?;

                let mut games = self.live_games.lock().unwrap();
                games.insert(game_id.clone(), new_game);

                Ok(game_id)
            }
        }
    }
}

impl Default for House {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HTTP Route Handlers
// ============================================================================

/// Health check / ping endpoint
#[get("/")]
pub async fn index() -> Json<ServerResponse> {
    Json(ServerResponse::success("Poker server is running"))
}

/// Returns the house rules for all games.
#[get("/rules")]
pub async fn get_rules() -> Json<HouseRules> {
    let rules = HouseRules {
        min_bet: 1,
        max_bet: 65535,
        max_raises_per_round: 3,
        starting_chips: 500,
        ante: 100,
        small_blind: 50,
        big_blind: 100,
    };
    Json(rules)
}

/// Returns a list of all games, including those waiting for players
/// and those currently in-progress.
#[get("/games")]
pub async fn list_games(house: &State<House>) -> Json<GameListResponse> {
    let games = house.live_games.lock().unwrap();
    let mut game_list = Vec::new();

    for (id, game) in games.iter() {
        let summary = GameSummary {
            game_id: id.clone(),
            game_type: game.get_game_type(),
            player_count: game.get_player_count(),
            max_players: game.get_max_players(),
            status: game.get_status(),
            pot: game.get_pot(),
        };
        game_list.push(summary);
    }

    Json(GameListResponse { games: game_list })
}

/// Creates a new game and adds the requesting player to it.
///
/// Request body should contain:
///     - player_id: String
///     - username: String
///     - game_type: GameType
#[post("/games", format = "json", data = "<request>")]
pub async fn create_game(
    request: Json<CreateGameRequest>,
    house: &State<House>,
    db: &State<DatabaseConnection>
) -> Json<GameResponse> {
    let inner_request = request.into_inner();

    // TODO: Fetch player from database to get their chip balance
    // For now, create player with default starting chips
    let player = Player::new(inner_request.username.clone(), 500);

    let mut new_game = match house.create_new_game(inner_request.game_type) {
        Ok(game) => game,
        Err(e) => return Json(GameResponse::error(format!("Failed to create game: {}", e))),
    };

    let game_id = new_game.get_game_id().to_string();

    match new_game.add_player(player) {
        Ok(_) => {
            let mut games = house.live_games.lock().unwrap();
            games.insert(game_id.clone(), new_game);

            // TODO: Build proper GameStateUpdate from game state
            let message = format!("Game created successfully. Waiting for players in game {}", game_id);
            Json(GameResponse {
                success: true,
                message,
                game_id: Some(game_id),
                game_state: None, // TODO: Add game state
            })
        }
        Err(e) => Json(GameResponse::error(format!("Failed to add player to game: {}", e))),
    }
}

/// Adds a player to an existing game.
///
/// Request body should contain:
///     - player_id: String
///     - username: String
///     - game_id: String
#[post("/games/<game_id>/join", format = "json", data = "<request>")]
pub async fn join_game(
    game_id: String,
    request: Json<JoinGameRequest>,
    house: &State<House>,
) -> Json<GameResponse> {
    let inner_request = request.into_inner();

    // TODO: Fetch player from database to get their chip balance
    let player = Player::new(inner_request.username.clone(), 500);

    let result = house.lock_and_add(player, None, Some(game_id.clone()));

    match result {
        Ok(joined_game_id) => {
            let message = format!("Successfully joined game {}", joined_game_id);
            // TODO: Build proper GameStateUpdate from game state
            Json(GameResponse {
                success: true,
                message,
                game_id: Some(joined_game_id),
                game_state: None, // TODO: Add game state
            })
        }
        Err(e) => Json(GameResponse::error(format!("Failed to join game: {}", e))),
    }
}

/// Returns the current state of a specific game.
///
/// Query parameters:
///     - player_id: String (required, to provide player-specific view)
#[get("/games/<game_id>?<player_id>")]
pub async fn get_game(
    game_id: String,
    player_id: String,
    house: &State<House>,
) -> Option<Json<GameStateUpdate>> {
    let games = house.live_games.lock().unwrap();

    games.get(&game_id).map(|game| {
        // TODO: Build proper GameStateUpdate from game state
        // This should include player-specific information like their hand
        let state = build_game_state_update(game, Some(&player_id));
        Json(state)
    })
}

/// Performs a game action (fold, check, call, bet, raise, draw).
///
/// Request body should contain:
///     - player_id: String
///     - game_id: String
///     - action: GameAction
#[post("/games/<_game_id>/action", format = "json", data = "<_request>")]
pub async fn perform_action(
    _game_id: String,
    _request: Json<ActionRequest>,
    _house: &State<House>,
) -> Json<GameResponse> {

    // TODO: Parse player_id from String to Uuid
    // TODO: Look up game and perform action
    // TODO: Return updated game state

    // Placeholder implementation
    Json(GameResponse::error("Action handling not yet implemented".to_string()))
}

/// Returns statistics for a specific player.
///
/// TODO: This should query the database for player stats
#[get("/players/<player_id>/stats")]
pub async fn get_stats(player_id: String) -> Json<PlayerStats> {
    // TODO: Query database for player stats
    // Placeholder response
    Json(poker_core::PlayerStats {
        player_id,
        username: "Unknown".to_string(),
        rounds_played: 0,
        pots_won: 0,
        folds: 0,
        total_winnings: 0,
        current_balance: 0,
    })
}

/// Adds chips to a player's account.
///
/// This validates that the player can "afford" the chips using a credit system
/// where all players have a credit limit of 65,535.
///
/// Request body should contain:
///     - player_id: String
///     - num_chips: u32
///     - credit_limit: u32
#[post("/players/<player_id>/addchips", format = "json", data = "<_request>")]
pub async fn add_chips(
    player_id: String,
    _request: Json<AddChipsRequest>,
) -> Json<AddChipsResponse> {

    // TODO: Parse player_id String to Uuid
    // TODO: Fetch user from database
    // TODO: Validate credit limit
    // TODO: Update token balance

    // Placeholder implementation
    let player_uuid = Uuid::parse_str(&player_id).unwrap_or_default();
    Json(AddChipsResponse::error(
        "Add chips not yet implemented",
        player_uuid,
        65535,
    ))
}

/// Registers a viewer for a game.
///
/// TODO: Decide if we need to register viewers or just send them game state updates
#[post("/games/<_game_id>/viewers", format = "json", data = "<_request>")]
pub async fn register_viewer(
    _game_id: String,
    _request: Json<poker_core::ViewerRequest>,
) -> Json<ServerResponse> {
    // TODO: Implement viewer registration
    Json(ServerResponse::error("Viewer registration not yet implemented"))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Builds a GameStateUpdate from the current game state.
///
/// This converts internal game representation to the protocol type
/// that can be sent to clients.
///
/// TODO: Implement proper conversion logic
fn build_game_state_update(game: &Game, _player_id: Option<&str>) -> GameStateUpdate {
    // Placeholder implementation
    GameStateUpdate {
        game_id: game.get_game_id().to_string(),
        game_type: game.get_game_type(),
        pot: game.get_pot(),
        current_bet: 0, // TODO: Get from game state
        betting_round: poker_core::BettingRound::PreDraw, // TODO: Get from game state
        action_on: None, // TODO: Get from game state
        player_count: game.get_player_count(),
        players: vec![], // TODO: Build player info list
        community_cards: vec![], // TODO: For Texas Hold'em
        your_hand: vec![], // TODO: Get player's hand if player_id provided
        your_chips: 0, // TODO: Get player's chips
    }
}

/// Converts a Player to PlayerInfo for protocol communication.
///
/// PlayerInfo hides the player's actual cards and only shows card count.
fn player_to_info(player: &Player, is_dealer: bool) -> PlayerInfo {
    PlayerInfo {
        username: player.username.clone(),
        chips: player.chips,
        current_bet: player.current_bet,
        folded: player.is_folded,
        is_dealer,
        cards_count: player.hand.len(),
    }
}

// ============================================================================
// Action Handler Stubs
// ============================================================================
// These functions will be called by perform_action based on the action type.
// They need to be implemented with proper game logic.

/// Handles a fold action.
async fn fold(_game_id: String, _player_id: Uuid) -> Result<(), String> {
    // TODO: Implement fold logic
    Err("Not implemented".to_string())
}

/// Handles a check action.
async fn check(_game_id: String, _player_id: Uuid) -> Result<(), String> {
    // TODO: Implement check logic
    Err("Not implemented".to_string())
}

/// Handles a call action.
async fn call(_game_id: String, _player_id: Uuid) -> Result<(), String> {
    // TODO: Implement call logic
    Err("Not implemented".to_string())
}

/// Handles a bet action.
async fn bet(_game_id: String, _player_id: Uuid, _amount: u32) -> Result<(), String> {
    // TODO: Implement bet logic
    Err("Not implemented".to_string())
}

/// Handles a raise action.
async fn raise(_game_id: String, _player_id: Uuid, _amount: u32) -> Result<(), String> {
    // TODO: Implement raise logic
    Err("Not implemented".to_string())
}

/// Handles a draw action (for Five Card Draw).
async fn draw(_game_id: String, _player_id: Uuid, _discard_indices: Vec<usize>) -> Result<(), String> {
    // TODO: Implement draw logic
    Err("Not implemented".to_string())
}
