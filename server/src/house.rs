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
    // game specified by that id. On success, invalidates the search group, 
    // removing the game with incremented player count and relocating it to target. 
    //
    // If no optional parameter is supplied, finds the first free game with minimal 
    // player count and adds the player to it. 
    //
    // Parameters: 
    //      Player    - the player to add 
    //      search    - the group of games to search
    //      game_type - the type of game requested by the player
    //      game_id   - optional, if adding the player to a specific game
    //
    // Returns: 
    //      Result<String, &'static String> 
    pub fn lock_and_add(&mut self, player: &Player, 
                                   search: &Arc<Mutex<HashMap<String, Game>>>, 
                                   game_type: String, 
                                   game_id Option<String>) Result<String, &'static String> {
        match game_id { 
           Some(value) => {
                    let search_group = search.clone(); 
                    let mut locked_search_group = search_group.lock.unwrap();
                    match locked_search_group.get(game_id) { 
                        Some(game) => {
                            match game.table.seat_player(&player) { 
                                Ok(()) => { 
                                    let target_group = search.clone(); 
                                    let mut locked_target_group = search_group.lock.unwrap(); 
                                    locked_target_group.insert(game_id, game); 
                                    return Ok(game.game_id) 
                                }, 
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
                                Ok(()) => {
                                    let target_group = search.clone(); 
                                    let mut locked_target_group = target_group.lock.unwrap(); 
                                    locked_target_group.push(game); 
                                    return Ok(game.game_Id) 
                                },
                                Err(_) => { continue } 
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
    pub fn remove_player_from(game: &mut Game, player: &Player, target: &Arc<Mutex<HashMap<String, Game>>>) {
    }

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
    pub fn find_player_an_open_table(&mut self, player: &Player, game_type: String) Result<Uuid, &'static String>{
        let games = &House.live_games; 
        let result = lock_and_add(player, &House.live_games, game_type); 
        match result { 
            Ok(added) =>  {
                Ok(result)
            }
            Err(error) => {
                    //if no game of the requested type is found, create a new one
                    //and add the player to it
                    let mut new_game = self.create_new(game_type); 
                    new_game.table.seat_player(player); 
                    let mut locked_games = games.clone()
                                                .lock()
                                                .unwrap(); 
                    locked_games.push(new_game);  
            }
        }
}


///
///
///
///
///
///
#[post("/players/<player_id>/stats", format = "json", data = "<PlayerStats>")] 
async fn get_stats(player_id: &str) -> Json { 
} 

/// 
///
///
///
///
///
#[post("/games/<game_id>/viewer", format = "json")] 
async fn viewers(game_id: &str) { 
}

///
///
///
///
///
///
#[post("/rules")] 
pub async fn rules() { 
}

///
///
///
///
///
///
#[post("/games/<game_id>/action", format = "json", data = "<ActionRequest>")]
async fn perform_action(request: ActionRequest) {
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

///
///
///
#[post("/games")] 
async fn create_game() { 

}

///
///
///
#[get("/games")] 
async fn list_games() {
    let mut response = GameListResponse::new(); 
    let search_group = search.clone(); 
    let mut locked_search_group = search_group.lock.unwrap();
    for (id, game) in locked_search_group.iter_mut() { 
        let mut summary = GameSummary::new(); 
        summary.game_id = id;
        summary.game_type = game.game_type; 
        summary.player_count = game.table.len(); 
        summary.max_players = 5; 
        summary.status = InProgress; 
        //TODO summary.status + 
    }
}


///
///
///
///
///
#[post("/games/<game_id>/join", format = "json", data = "<JoinGameRequest>")]
async fn join_game(request: Json<JoinGameRequest>){
    let inner_request = request.into_inner(); 

}


///
///
///
#[post("/games", format = "json", data = "<CreateGameRequest")] 
async fn create_game(request: Json<CreateGameRequest>){ 
    let inner_request = request.into_inner(); 
}

///
///
///
#[get("/games/<game_id>")] 
pub async get_game(game_id: &str) { 
    let mut GameStateUpdate = GameStateUpdate::new(); 
    match game_id {
        Some(id) if id
    }
}
