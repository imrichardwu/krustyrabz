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
use poker_core::hand::Hand;
use crate::player::Player;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rocket::{get, post, State};
use rocket::serde::{json::Json, Deserialize};

use storage::entities::user_account::Model;
use storage::repository::Repository;

use crate::game::{FiveCardDraw, SevenCardStud, TexasHoldEm};
use crate::betting::BettingRound as ServerBettingRound;
use poker_core::protocol::GameAction;
use tokio::sync::broadcast;
use rocket::response::stream::{Event, EventStream};

// ============================================================================
// House Structure
// ============================================================================

/// This data structure stores all currently live and pending games.
/// Games are stored in a HashMap keyed by game_id (as String for protocol compatibility).
#[derive(Debug)]
pub struct House {
    pub live_games: Arc<Mutex<HashMap<String, Game>>>,
    /// Optional cache of players' hands keyed by player_id string.
    /// Currently populated at game creation; can be expanded later.
    pub hands: Arc<Mutex<HashMap<String, Hand>>>,
    /// Per-game broadcast channels for SSE push updates.
    pub event_senders: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl House {
    pub fn new() -> Self {
        Self {
            live_games: Arc::new(Mutex::new(HashMap::new())),
            hands: Arc::new(Mutex::new(HashMap::new())),
            event_senders: Arc::new(Mutex::new(HashMap::new())),
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

    /// Creates a broadcast channel for the given game, used to push SSE updates.
    pub fn create_event_channel(&self, game_id: &str) {
        let (tx, _) = broadcast::channel(32);
        self.event_senders.lock().unwrap().insert(game_id.to_string(), tx);
    }

    /// Broadcasts the current public game state to all SSE subscribers of a game.
    pub fn broadcast_game_state(&self, game_id: &str) {
        let json = {
            let games = self.live_games.lock().unwrap();
            games.get(game_id)
                .and_then(|game| serde_json::to_string(&build_game_state_update(game, None)).ok())
        };
        if let Some(json) = json {
            let senders = self.event_senders.lock().unwrap();
            if let Some(tx) = senders.get(game_id) {
                let _ = tx.send(json);
            }
        }
    }

    /// Helper method to remove an empty game.
    ///
    /// Parameters:
    ///     game_id   - The game to remove from the house
    ///
    /// Returns:
    ///     Result<(), String>
    pub fn remove_game(&self, game_id: &str) -> Result<(), String> {
        let mut games = self.live_games.lock().unwrap();
        games.remove(game_id).ok_or(format!("Failed to remove game: {}", game_id))?;
        drop(games);
        self.event_senders.lock().unwrap().remove(game_id);
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
                if game.is_empty() {
                    games.remove(game_id);
                }
                Ok(())
            }
            None => Err(format!("Game not found: {}", game_id)),
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
                drop(games);
                self.create_event_channel(&game_id);

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

/// Request body for starting a hand (Five Card Draw).
#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct StartHandRequest {
    pub player_id: String,
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
) -> Json<GameResponse> {
    let inner_request = request.into_inner();
    let player_id = match Uuid::parse_str(&inner_request.player_id) {
        Ok(id) => id,
        Err(_) => return Json(GameResponse::error("Invalid player_id".to_string())),
    };

    let repo = match Repository::new().await {
        Ok(r) => r,
        Err(e) => return Json(GameResponse::error(format!("Failed to create repository: {}", e))),
    };

    let model = match repo.get_user_by_id(player_id).await {
        Ok(m) => m,
        Err(e) => return Json(GameResponse::error(format!("User not found: {}", e))),
    };

    let player = model_to_player(&model);

    let mut new_game = match house.create_new_game(inner_request.game_type) {
        Ok(game) => game,
        Err(e) => return Json(GameResponse::error(format!("Failed to create game: {}", e))),
    };

    let game_id = new_game.get_game_id().to_string();

    match new_game.add_player(player) {
        Ok(_) => {
            let state = {
                let mut games = house.live_games.lock().unwrap();
                games.insert(game_id.clone(), new_game);
                let mut hands = house.hands.lock().unwrap();
                hands.insert(player_id.to_string(), Hand::new());
                let game = games.get(&game_id).expect("just inserted");
                build_game_state_update(game, Some(&inner_request.player_id))
            }; // games and hands locks released
            house.create_event_channel(&game_id);
            let message = format!(
                "Game created successfully. Waiting for players in game {}",
                game_id
            );
            Json(GameResponse {
                success: true,
                message,
                game_id: Some(game_id),
                game_state: Some(state),
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

    let player = match get_player_from_db(&inner_request.player_id).await {
        Ok(p) => p,
        Err(e) => return Json(GameResponse::error(format!("Failed to load player: {}", e))),
    };

    let result = house.lock_and_add(player, None, Some(game_id.clone()));

    match result {
        Ok(joined_game_id) => {
            let state = {
                let games = house.live_games.lock().unwrap();
                let game = match games.get(&joined_game_id) {
                    Some(g) => g,
                    None => {
                        return Json(GameResponse::error("Game not found after join".to_string()));
                    }
                };
                build_game_state_update(game, Some(&inner_request.player_id))
            }; // games lock released
            house.broadcast_game_state(&joined_game_id);
            let message = format!("Successfully joined game {}", joined_game_id);
            Json(GameResponse {
                success: true,
                message,
                game_id: Some(joined_game_id),
                game_state: Some(state),
            })
        }
        Err(e) => Json(GameResponse::error(format!("Failed to join game: {}", e))),
    }
}

#[post("/games/<game_id>/leave", format = "json", data = "<request>")]
pub async fn leave_game(
    game_id: String,
    request: Json<JoinGameRequest>,
    house: &State<House>,
) -> Json<GameResponse> {
    let inner_request = request.into_inner();
    let player_id = match Uuid::parse_str(&inner_request.player_id) {
        Ok(id) => id,
        Err(_) => return Json(GameResponse::error("Invalid player_id".to_string())),
    };

    match house.remove_player(&game_id, player_id) {
        Ok(_) => {
            let game_state = {
                let games = house.live_games.lock().unwrap();
                games.get(&game_id).map(|game| build_game_state_update(game, None))
            }; // games lock released
            house.broadcast_game_state(&game_id);
            let message = format!("Left game {}", game_id);
            Json(GameResponse {
                success: true,
                message,
                game_id: Some(game_id),
                game_state,
            })
        }
        Err(e) => Json(GameResponse::error(format!("Failed to leave game: {}", e))),
    }
}

/// Starts a hand (Five Card Draw only). Deals 5 cards to each player and transitions to PreDraw betting.
///
/// Request body: `{ "player_id": "..." }` (player must be in the game).
#[post("/games/<game_id>/start", format = "json", data = "<request>")]
pub async fn start_hand(
    game_id: String,
    request: Json<StartHandRequest>,
    house: &State<House>,
) -> Json<GameResponse> {
    let player_id_str = request.player_id.clone();
    let _player_id = match Uuid::parse_str(&player_id_str) {
        Ok(id) => id,
        Err(_) => return Json(GameResponse::error("Invalid player_id".to_string())),
    };

    let outcome: Result<GameStateUpdate, String> = {
        let mut games = house.live_games.lock().unwrap();
        let game = match games.get_mut(&game_id) {
            Some(g) => g,
            None => return Json(GameResponse::error("Game not found".to_string())),
        };
        game.start_hand().map(|()| build_game_state_update(game, Some(&player_id_str)))
    }; // games lock released

    match outcome {
        Ok(state) => {
            house.broadcast_game_state(&game_id);
            Json(GameResponse::success("Hand started".to_string(), game_id, state))
        }
        Err(e) => Json(GameResponse::error(e)),
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
#[post("/games/<game_id>/action", format = "json", data = "<request>")]
pub async fn perform_action(
    game_id: String,
    request: Json<ActionRequest>,
    house: &State<House>,
) -> Json<GameResponse> {
    let inner = request.into_inner();
    let player_id = match Uuid::parse_str(&inner.player_id) {
        Ok(id) => id,
        Err(_) => return Json(GameResponse::error("Invalid player_id".to_string())),
    };
    if inner.game_id != game_id {
        return Json(GameResponse::error("game_id in URL and body must match".to_string()));
    }

    let outcome: Result<GameStateUpdate, String> = {
        let mut games = house.live_games.lock().unwrap();
        let game = match games.get_mut(&game_id) {
            Some(g) => g,
            None => return Json(GameResponse::error("Game not found".to_string())),
        };
        let result = match game.get_game_type() {
            poker_core::GameType::FiveCardDraw => match &inner.action {
                GameAction::Fold => handle_fold(game, player_id),
                GameAction::Check => handle_check(game, player_id),
                GameAction::Call => handle_call(game, player_id),
                GameAction::Bet { amount } => handle_bet(game, player_id, *amount),
                GameAction::Raise { amount } => handle_raise(game, player_id, *amount),
                GameAction::Draw { discard_indices } => handle_draw(game, player_id, discard_indices.clone()),
                GameAction::AllIn => Err("AllIn not supported for Five Card Draw".to_string()),
            },
            poker_core::GameType::SevenCardStud | poker_core::GameType::TexasHoldEm => {
                game.handle_action(player_id, inner.action.clone())
            }
        };
        result.map(|()| build_game_state_update(game, Some(&inner.player_id)))
    }; // games lock released

    match outcome {
        Ok(state) => {
            house.broadcast_game_state(&game_id);
            Json(GameResponse::success("Action applied".to_string(), game_id, state))
        }
        Err(e) => Json(GameResponse::error(e)),
    }
}

/// Returns statistics for a specific player.
#[get("/players/<player_id>/stats")]
pub async fn get_stats(player_id: String) -> Json<PlayerStats> {
    let player_uuid = match Uuid::parse_str(&player_id) {
        Ok(id) => id,
        Err(_) => {
            return Json(poker_core::PlayerStats {
                player_id,
                username: "Unknown".to_string(),
                rounds_played: 0,
                pots_won: 0,
                folds: 0,
                total_winnings: 0,
                current_balance: 0,
            });
        }
    };

    let repo = match Repository::new().await {
        Ok(r) => r,
        Err(_) => {
            return Json(poker_core::PlayerStats {
                player_id,
                username: "Unknown".to_string(),
                rounds_played: 0,
                pots_won: 0,
                folds: 0,
                total_winnings: 0,
                current_balance: 0,
            });
        }
    };

    let model = match repo.get_user_by_id(player_uuid).await {
        Ok(m) => m,
        Err(_) => {
            return Json(poker_core::PlayerStats {
                player_id,
                username: "Unknown".to_string(),
                rounds_played: 0,
                pots_won: 0,
                folds: 0,
                total_winnings: 0,
                current_balance: 0,
            });
        }
    };

    let rounds_played = model.rounds_played.unwrap_or(0) as u32;
    let pots_won = model.pots_won.unwrap_or(0) as u32;
    let folds = model.number_folds.unwrap_or(0) as u32;
    let current_balance = model.token_balance.unwrap_or(0.0) as u32;

    Json(poker_core::PlayerStats {
        player_id,
        username: model.username,
        rounds_played,
        pots_won,
        folds,
        total_winnings: 0, // not stored in UserAccount
        current_balance,
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
#[post("/players/<player_id>/addchips", format = "json", data = "<request>")]
pub async fn add_chips(
    player_id: String,
    request: Json<AddChipsRequest>,
) -> Json<AddChipsResponse> {
    let player_uuid = match Uuid::parse_str(&player_id) {
        Ok(id) => id,
        Err(_) => {
            return Json(AddChipsResponse::error(
                "Invalid player_id",
                Uuid::nil(),
                65535,
            ));
        }
    };

    let num_chips = request.num_chips;
    let credit_limit = request.credit_limit;

    let repo = match Repository::new().await {
        Ok(r) => r,
        Err(e) => {
            return Json(AddChipsResponse::error(
                format!("Failed to create repository: {}", e),
                player_uuid,
                credit_limit,
            ));
        }
    };

    let model = match repo.get_user_by_id(player_uuid).await {
        Ok(m) => m,
        Err(e) => {
            return Json(AddChipsResponse::error(
                format!("User not found: {}", e),
                player_uuid,
                credit_limit,
            ));
        }
    };

    let current_balance = model.token_balance.unwrap_or(0.0) as u32;
    if current_balance + num_chips > credit_limit {
        return Json(AddChipsResponse::error(
            "Would exceed credit limit",
            player_uuid,
            credit_limit,
        ));
    }

    match repo
        .update_user_token_balance(player_uuid, num_chips as f64)
        .await
    {
        Ok(_) => Json(AddChipsResponse::success(
            "Chips added",
            player_uuid,
            credit_limit,
            num_chips,
        )),
        Err(e) => Json(AddChipsResponse::error(
            format!("Failed to update balance: {}", e),
            player_uuid,
            credit_limit,
        )),
    }
}

/// Registers a viewer for a game.
///
/// Minimal implementation: returns success without persisting (for five-card testing).
#[post("/games/<_game_id>/viewers", format = "json", data = "<_request>")]
pub async fn register_viewer(
    _game_id: String,
    _request: Json<poker_core::ViewerRequest>,
) -> Json<ServerResponse> {
    Json(ServerResponse::success("Viewer registered"))
}

/// SSE endpoint — browser connects once and receives a push event after every
/// game-state change (join, start hand, action, leave).
/// Each event payload is a JSON-serialised `GameStateUpdate` (public view, no private cards).
#[get("/games/<game_id>/events")]
pub async fn game_events(game_id: String, house: &State<House>) -> EventStream![] {
    let rx = house.event_senders.lock().unwrap()
        .get(&game_id)
        .map(|tx| tx.subscribe());
    EventStream! {
        if let Some(mut rx) = rx {
            loop {
                match rx.recv().await {
                    Ok(data) => yield Event::data(data),
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Maps server betting round to protocol (core) betting round.
fn to_protocol_betting_round(r: ServerBettingRound) -> poker_core::BettingRound {
    use poker_core::BettingRound as CoreRound;
    match r {
        ServerBettingRound::PreDeal    => CoreRound::PreDraw,
        ServerBettingRound::PreDraw    => CoreRound::PreDraw,
        ServerBettingRound::Drawing    => CoreRound::Drawing,
        ServerBettingRound::PostDraw   => CoreRound::PostDraw,
        ServerBettingRound::PreFlop    => CoreRound::PreFlop,
        ServerBettingRound::Flop       => CoreRound::Flop,
        ServerBettingRound::Turn       => CoreRound::Turn,
        ServerBettingRound::River      => CoreRound::River,
        ServerBettingRound::ThirdStreet   => CoreRound::ThirdStreet,
        ServerBettingRound::FourthStreet  => CoreRound::FourthStreet,
        ServerBettingRound::FifthStreet   => CoreRound::FifthStreet,
        ServerBettingRound::SixthStreet   => CoreRound::SixthStreet,
        ServerBettingRound::SeventhStreet => CoreRound::River, // 7th street maps to River in protocol
    }
}

/// Builds a GameStateUpdate from the current game state.
///
/// This converts internal game representation to the protocol type
/// that can be sent to clients.
fn build_game_state_update(game: &Game, player_id: Option<&str>) -> GameStateUpdate {
    let players = game.get_players();
    let dealer_idx = game.get_dealer_index();
    let betting_state = game.get_betting_state();
    let players_info: Vec<PlayerInfo> = players
        .iter()
        .enumerate()
        .map(|(i, p)| player_to_info(p, i == dealer_idx))
        .collect();

    let (your_hand, your_chips) = match player_id.and_then(|id| Uuid::parse_str(id).ok()) {
        Some(uid) => {
            let hand_cards = players
                .iter()
                .find(|p| p.id == uid)
                .map(|p| {
                    p.hand
                        .cards()
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let chips = players
                .iter()
                .find(|p| p.id == uid)
                .map(|p| p.chips)
                .unwrap_or(0);
            (hand_cards, chips)
        }
        None => (Vec::new(), 0),
    };

    let action_on_username = game
        .get_action_on()
        .and_then(|uid| players.iter().find(|p| p.id == uid))
        .map(|p| p.username.clone());

    let last_hand_message = game.get_last_showdown().map(|winners| {
        if winners.is_empty() {
            "Hand over (no winners).".to_string()
        } else if winners.len() == 1 {
            format!("*** SHOWDOWN: {} won ${} ***", winners[0].0, winners[0].1)
        } else {
            let parts: Vec<String> = winners
                .iter()
                .map(|(name, amt)| format!("{} won ${}", name, amt))
                .collect();
            format!("*** SHOWDOWN (tie): {} ***", parts.join(", "))
        }
    });

    GameStateUpdate {
        game_id: game.get_game_id().to_string(),
        game_type: game.get_game_type(),
        pot: game.get_pot(),
        current_bet: betting_state.to_call,
        betting_round: to_protocol_betting_round(game.get_betting_round()),
        action_on: action_on_username,
        player_count: game.get_player_count(),
        players: players_info,
        community_cards: game.get_community_cards(),
        your_hand,
        your_chips,
        last_hand_message,
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

/// Converts a Model to a Player for protocol communication. 
fn model_to_player(model: &Model) -> Player { 
    let token_balance = match model.token_balance { 
        Some(val) => val as u32, 
        None => 0, 
    }; 
    Player { 
        id: model.id,
        username: model.username.clone(),  
        chips: token_balance,
        hand: Hand::new(), 
        is_folded: false, 
        game_id: model.game_id,
        current_bet: 0,
    }
}

/// Gets a user account model from the database and converts it to a Player.
///
/// This is a small convenience wrapper around `Repository::get_user_by_id`.
async fn get_player_from_db(player_id: &str) -> Result<Player, String> {
    let player_uuid = Uuid::parse_str(player_id).map_err(|e| e.to_string())?;

    let repo = Repository::new()
        .await
        .map_err(|e| format!("Failed to create repository: {e}"))?;

    let model = repo
        .get_user_by_id(player_uuid)
        .await
        .map_err(|e| format!("Failed to get user: {e}"))?;

    Ok(model_to_player(&model))
}

/// Gets the specified player from their current in-memory game, if any.
///
/// Iterates over all live games in the `House` and returns the first matching player.
#[allow(dead_code)]
async fn get_player(player_id: &str, house: &State<House>) -> Option<Player> {
    let games = house.live_games.lock().unwrap();

    for game in games.values() {
        if let Some(player) = find_player_in_game(player_id, game) {
            return Some(player);
        }
    }

    None
}

/// Checks if a player is in the given game and returns them if found.
#[allow(dead_code)]
fn find_player_in_game(player_id: &str, game: &Game) -> Option<Player> {
    game.get_players()
        .into_iter()
        .find(|player| player.id.to_string() == player_id)
}

// ============================================================================
// Action Handlers (Five Card Draw for now)
// ============================================================================
// These functions are called by perform_action. They dispatch to the game's
// betting or draw logic based on the current phase.

/// Handles a fold action (PreDraw or PostDraw betting).
fn handle_fold(game: &mut Game, player_id: Uuid) -> Result<(), String> {
    match game {
        Game::FiveCardDraw(g) => match g.betting_round {
            ServerBettingRound::PreDeal => Err("game_not_started".to_string()),
            ServerBettingRound::PreDraw => g.predraw_betting(player_id, GameAction::Fold),
            ServerBettingRound::PostDraw => g.postdraw_betting(player_id, GameAction::Fold),
            ServerBettingRound::Drawing => Err("cannot_fold_in_draw_phase".to_string()),
            _ => Err("wrong_phase".to_string()),
        },
        _ => Err("only Five Card Draw supported".to_string()),
    }
}

/// Handles a check action (PreDraw or PostDraw betting).
fn handle_check(game: &mut Game, player_id: Uuid) -> Result<(), String> {
    match game {
        Game::FiveCardDraw(g) => match g.betting_round {
            ServerBettingRound::PreDeal => Err("game_not_started".to_string()),
            ServerBettingRound::PreDraw => g.predraw_betting(player_id, GameAction::Check),
            ServerBettingRound::PostDraw => g.postdraw_betting(player_id, GameAction::Check),
            ServerBettingRound::Drawing => Err("cannot_check_in_draw_phase".to_string()),
            _ => Err("wrong_phase".to_string()),
        },
        _ => Err("only Five Card Draw supported".to_string()),
    }
}

/// Handles a call action (PreDraw or PostDraw betting).
fn handle_call(game: &mut Game, player_id: Uuid) -> Result<(), String> {
    match game {
        Game::FiveCardDraw(g) => match g.betting_round {
            ServerBettingRound::PreDeal => Err("game_not_started".to_string()),
            ServerBettingRound::PreDraw => g.predraw_betting(player_id, GameAction::Call),
            ServerBettingRound::PostDraw => g.postdraw_betting(player_id, GameAction::Call),
            ServerBettingRound::Drawing => Err("cannot_call_in_draw_phase".to_string()),
            _ => Err("wrong_phase".to_string()),
        },
        _ => Err("only Five Card Draw supported".to_string()),
    }
}

/// Handles a bet action (PreDraw or PostDraw betting).
fn handle_bet(game: &mut Game, player_id: Uuid, amount: u32) -> Result<(), String> {
    match game {
        Game::FiveCardDraw(g) => match g.betting_round {
            ServerBettingRound::PreDeal => Err("game_not_started".to_string()),
            ServerBettingRound::PreDraw => {
                g.predraw_betting(player_id, GameAction::Bet { amount })
            }
            ServerBettingRound::PostDraw => {
                g.postdraw_betting(player_id, GameAction::Bet { amount })
            }
            ServerBettingRound::Drawing => Err("cannot_bet_in_draw_phase".to_string()),
            _ => Err("wrong_phase".to_string()),
        },
        _ => Err("only Five Card Draw supported".to_string()),
    }
}

/// Handles a raise action (PreDraw or PostDraw betting).
fn handle_raise(game: &mut Game, player_id: Uuid, amount: u32) -> Result<(), String> {
    match game {
        Game::FiveCardDraw(g) => match g.betting_round {
            ServerBettingRound::PreDeal => Err("game_not_started".to_string()),
            ServerBettingRound::PreDraw => {
                g.predraw_betting(player_id, GameAction::Raise { amount })
            }
            ServerBettingRound::PostDraw => {
                g.postdraw_betting(player_id, GameAction::Raise { amount })
            }
            ServerBettingRound::Drawing => Err("cannot_raise_in_draw_phase".to_string()),
            _ => Err("wrong_phase".to_string()),
        },
        _ => Err("only Five Card Draw supported".to_string()),
    }
}

/// Handles a draw action (Five Card Draw drawing phase).
fn handle_draw(
    game: &mut Game,
    player_id: Uuid,
    discard_indices: Vec<usize>,
) -> Result<(), String> {
    match game {
        Game::FiveCardDraw(g) => g.handle_draw_action(player_id, discard_indices),
        _ => Err("only Five Card Draw supported".to_string()),
    }
}
