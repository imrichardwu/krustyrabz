pub mod game;
pub use game::Game;
use uuid::Uuid; 
use crate::client::{Message};
use poker_core::{ ActionRequest, CreateGameRequest, GameAction, GameListResponse, GameResponse, 
    GameStateUpdate, GameStatus, GameType, HouseRules, JoinGameRequest, PlayerStats, ServerResponses, StatsRequest, ViewerRequest
}; 
use std::sync::{Arc, Mutex};
use std::collections::HashMap; 
//use future_util::{SinkExt, StreamExt}; 
use rocket::{get, launch, routes, State, Shutdown}; 
use rocket::serde::{Serialize, Deserialize, json::Json}};
//use rocket::response::stream::{EventStream, Event}; 
//use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
//use rocket::tokio::select; 
//use rocket::form::Form; 
use crate::storage::Repository; 

/// This data structure stores all currently live and pending games
/// grouped by player count.
///
#[derive(Serialize, Debug)]
pub struct House { 
    pub live_games:     Arc<Mutex<HashMap<String, Game>>>; 
}

impl House {
    pub fn new() -> Self {
        Self { 
            let live_games:     Arc::new(Mutex::new(HashMap::new()), 
        }
    }

    // Helper method, creates a game of the specified type. 
    //
    // Parameters: 
    //      GameType - the type of game to create 
    //
    // Returns: 
    //      None 
    pub fn create_new(variant : Game) -> Result<Game, 'static String> { 
        match variant { 
            Game::FiveCardDraw(_)  => { 
                let game = FiveCardDraw::new()?; 
                Ok(Game::FiveCardDraw(game))
            },
            _ => Err("Failed to create new game of Five Card Draw"), 
            Game::SevenCardStud(_) => {
                let game = SevenCardStud::new()?; 
                Ok(Game::SevenCardStud(game))
            },
            _ => Err("Failed to create new game of Seven Card Stud"), 
            Game::TexasHoldEm(_) => { 
                let game = TexasHoldEm::new()?; 
                Ok(Game::TexasHoldEm(game))
            },
            _ => Err("Failed to create new game of Texas Hold 'Em")
        }
    }

    // Helper method, locks the live_games data structure specified by search and
    // attempts to locate a free game inside it. 
    //
    // If optional parameter game_id is supplied, attempts to add a player to the 
    // game specified by that id. 
    //
    //Parameters: 
    //      Player    - the player to add 
    //      search    - the group of games to search
    //      game_type - the type of game requested by the player
    //      game_id   - optional, if adding the player to a specific game
    //
    // Returns: 
    //      Result<String, &'static String> 
    pub fn lock_and_add(&mut self, player: &Player, 
                                   search: &Arc<Mutex<HashMap<String, Game>>>, 
                                   game_type: Option<String>, 
                                   game_id Option<String>) Result<Game, &'static String> {
        match game_id { 
           Some(value) => {
                    let search_group = search.clone(); 
                    let mut locked_search_group = search_group.lock.unwrap();
                    match locked_search_group.get(game_id) { 
                        Some(game) => {
                            match game.table.seat_player(&player) { 
                                Ok(()) => return Ok(game), 
                                Err(_) => Err(format!("Failed adding player to game: {}", game_id));  
                            }
                        }, 
                        None => Err(format!("Failed to find game: {}", game_id) 
                    }
           }, 
           //if no game_id supplied, just join the first available game 
           None => {
                    let search_group = search.clone(); 
                    let mut locked_search_group = search_group.lock.unwrap();
                    for (id, game) in locked_search_group.iter_mut() { 
                        if game.game_type.to_string() == game_type
                           && game.len() < 5{ 
                            match game.table.seat_player(&player) { 
                                Ok(()) => return Ok(game),
                                Err(_) => continue 
                        }
                    }
                }
            }
       } 
       return Err(format!("Failed adding player {} to game of type {}", player.id, game_type));
    }

    // Helper method, on client disconnect, removes the dropped player from the game 
    // and relocates that game to the group with decremented player counts. 
    //
    // Parameters: 
    //      player  - the player to remove from the table 
    //      game    - the game to remove the player from
    //      target  - the group to move the game into after reducing its player count
    pub fn remove_player_from(game: &mut Game, player: &Player, target: &Arc<Mutex<HashMap<String, Game) {
    }

    //TODO could change to polling with timeout. this implementation is extremely simple and 
    //just gives up if no open game found, doesn't consider if another player 
    //added a new game after the check was completed
    /// Locks and searches the live_games data structure in an attempt to locate 
    /// an open game (i.e. one with < 5 active players). If none are found, it 
    /// creates a new game.
    ///
    /// Parameters:
    ///     player    - the player to add to a table 
    ///     game_type - the type of game requested by the player  
    ///
    /// Returns: 
    ///    Result<String, 'static String> 
    ///     
    pub fn find_player_an_open_table(&mut self, house: &State<House>, player: &Player, game_type: String) Result<Uuid, &'static String>{
        let games = &house.live_games; 
        let result = lock_and_add(player, games, game_type); 
        match result { 
            Ok(added) =>  {
                Ok(result)
            }
            Err(error) => {
                    //if no game of the requested type is found, create a new one
                    //and add the player to it
                    let mut new_game = self.create_new(game_type); 
                    new_game.core.table.seat_player(player); 
                    let mut locked_games = games.clone()
                                                .lock()
                                                .unwrap(); 
                    locked_games.insert(new_game.game_id, new_game);  
            }
        }
}

#[post("/players/<player_id>/stats", format = "json", data = "<PlayerStats>")] 
async fn get_stats(player_id: &str) -> Json<PlayerStats> {
    let player = db.get_user_by_id(player_id).await.ok()?; 
    let response =  PlayerStats { 
        player_id: player_id, 
        username: player.username, 
        chips: player.chips, 
        current_bet: player.current_bet, 
        folded: player.folded, 
        is_dealer: false, 
        cards_count: player.hand.len(), 
    }; 
    Json(response)
} 

#[post("/games/<game_id>/viewers", format = "json", data = "<ViewerRequest>")] 
async fn register_viewer(game_id: &str, request: Json<ViewerRequest>) -> Json<ViewerRequest> { 
//TODO do we need to register viewers? can we not just send them game state updates ? 
}


#[post("/rules")] 
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
    return Json(rules) 
}

///
///
///
#[post("/games/<game_id>/action", format = "json", data = "<ActionRequest>")]
async fn perform_action(request: Json<ActionRequest>) {
    let inner_request = request.into_inner(); 
    match inner_request.action { 
        GameAction::Fold  => fold(inner_request),
        GameAction::Check => check(inner_request), 
        GameAction::Call  => call(inner_request), 
        GameAction::Bet   => bet(inner_request), 
        GameAction::Raise => raise(inner_request), 
        GameAction::Draw  => draw(inner_request)  

    }
}

///
///
async fn fold(request: ActionRequest) { 
}

/// 
///
async fn check(request: ActionRequest) { 
}


///
///
async fn call(request: ActionRequest) { 
}

///
///
async fn bet(request: ActionRequest) {
}

///
///
async fn raise(request: ActionRequest) { 
}

///
///
async fn draw(request: ActionRequest) { 
}


/// Attempts to add the number of requested chips to the account belonging 
/// to the player specified in the request. Validates that a player can 
/// "afford" that number of chips using some unrealistic worldbuilding which 
/// gives all players credit limits of 65,535 and has them pay the balances 
/// in full after every gambling session. This means that a client's chip 
/// request will always be approved provided (token_balance + num_chips_requested) 
/// <= 65535.  
///
///
#[post("/players/<player_id>/addchips", format = "json", data = "<AddChipsRequest>")] 
async fn add_chips(request: Json<AddChipsRequest>, db: &State<DatabaseConnection>) -> Json<AddChipsResponse>{ 
    let inner_request = request.into_inner();
    let id = inner_request.player_id; 
    let num_chips = inner_request.num_chips; 
    let credit_limit = inner_request.credit_limit; 
    let user_account = db.get_user_by_id(id).await.ok()?; 
    let token_balance = user_account.token_balance; 
    let charge = (token_balance, num_chips); 
    match charge { 
        (balance, num_chips) if balance + num_chips > 65535 { 
            let message = "Error: insufficient funds. No chips were added to your account."; 
            let response = AddChipsResponse::error(message, id, credit_limit, 0); 
            return Json(response)
        }, 
        (balance, num_chips) if balance + num_chips <= 65535 { 
            let result = db.update_user_token_balance(id, num_chips); 
            match result { 
                Ok(model) => { 
                    let message = format!("Success: added {} chips to your account.", num_chips);
                    let response = AddChipsResponse::success(message, id, credit_limit, num_chips);
                    return Json(response)
                }, 
                Err() => { 
                    let message = "Error: user has sufficient funds, but the write to the database failed."; 
                    let response = AddChipsResponse::error(message, id, credit_limit, 0);
                    return Json(response); 
                }
            }
        }
    }
}

/// Returns a list of all games, including those waiting for players 
/// and those currently in-progress. 
///
#[get("/games")] 
async fn list_games() -> Json<GameListResponse> {
    let mut response = GameListResponse::new(); 
    let search_group = search.clone(); 
    let mut locked_search_group = search_group.lock.unwrap();
    for (id, game) in locked_search_group.iter_mut() { 
        let mut summary = GameSummary::new(); 
        summary.game_id = id;
        summary.game_type = game.game_type; 
        summary.player_count = game.table.len(); 
        summary.max_players = 5; 
        summary.status = GameStatus::WaitingForPlayers;
        summary.pot = game.pot; 
        response.push(summary); 
    }
    Json(response); 
}

/// Adds the player to the game specified by the request's game_id field. 
///
#[post("/games/<game_id>/join", format = "json", data = "<JoinGameRequest>")]
async fn join_game(request: Json<JoinGameRequest>, house: &State<House>){
    let inner_request = request.into_inner(); 
    
    let account = db.get_user_by_id(id).await().ok()?; 
    let mut player = Player::new(); 
    let game_id = inner_request.game_id; 
    player.id = inner_request.player_id; 
    player.username = inner_request.username; 
    player.game_id = game_id; 
    player.chips = account.token_balance; 

    let games = &house.live_games; 
    
    let result = lock_and_add(player, games, None, player.game_id); 
    match result { 
        Ok(game) => { 
            let message = format!("Success: now playing game {}", game_id); 
            let state = GameStateUpdate { 
                game_id: game.game_id, 
                game_type: Game::FiveCardDraw, 
                pot:  game.core.pot, 
                current_bet: 0, //TODO fix this 
                betting_round: game.betting_round, 
                action_on: game.core.action_on, 
                player_count: game.core.table.get_player_count(), 
                players: game.core.table.players, 
                community_cards = vec![],  //TODO fix this 
                your_hand = vec![], 
                your_chips = player.chips, 
            }; 
            let response = GameResponse::success(message, game_id, state);
            return Json(response); 
        }, 
        Err() => { 
            let message = format!("Error: unable to join game {}", game_id); 
            let response = GameResponse::error(message, game_id, None);
            return Json(response); 
        }
    }
}


/// Creates a new game and adds the player to it.
///
#[post("/games", format = "json", data = "<CreateGameRequest")] 
async fn create_game(request: Json<CreateGameRequest>, house: &State<House>) -> Json<GameResponse>{ 
    let inner_request = request.into_inner(); 
    let account = db.get_user_by_id(id).await().ok()?; 
    let mut player = Player {  
        player.id: inner_request.player_id, 
        player.username:  inner_request.username,  
        player.chips:  account.token_balance
    }; 

    let mut new_game = self.create_new(game_type); 
    new_game.table.seat_player(player);
    let games = &house.live_games; 
    let mut locked_games = games.clone()
                                .lock()
                                .unwrap(); 
    locked_games.insert(new_game.game_id, new_game); 

    let message = format!("Success: started new game. Waiting for players in game {}", game_id); 
    let response = GameResponse::success(message, new_game.game_id, None); 
    Json(response); 
}

/// Returns the current state of the game specified by game_id 
///
#[get("/games/<game_id>")] 
pub async get_game(game_id: &str, player_id: &str, house: &State<House>) -> Json<GameResponse>{ 
    let games = &house.live_games; 
    let mut locked_games = games.clone()
                                .lock()
                                .unwrap(); 
    let result = locked_games.get(game_id); 
    match result { 
        Some(game) => { 
            let state = GameStateUpdate { 
                game_id: game.game_id, 
                game_type: Game::FiveCardDraw, 
                pot:  game.core.pot, 
                current_bet: 0, //TODO fix this 
                betting_round: game.betting_round, 
                action_on: game.core.action_on, 
                player_count: game.core.table.get_player_count(), 
                players: game.core.table.players, 
                community_cards = vec![],  //TODO fix this 
                your_hand = vec![], 
                your_chips = player.chips, 
            }; 
            return Json(GameResponse::success("Found game", game_id, state)); 
        }, 
        None() => { 
            return Json(GameResponse::error("Couldn't find game", game_id, None));
        }
}
